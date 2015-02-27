#![feature(quote, plugin_registrar, rustc_private, plugin)]
#![feature(tempdir, path, io, fs, process)]

#![plugin(regex_macros)]

extern crate regex;

extern crate syntax;
extern crate rustc;

use std::path::{PathBuf, Path};
use std::fs::{self, TempDir, File};
use std::io;
use std::io::prelude::*;
use std::process::Command;

use syntax::ast;
use syntax::codemap;
use syntax::ext::base::{self, ExtCtxt, MacResult};
use syntax::parse::{self, token};
use rustc::plugin::Registry;

mod parser_any_macro;
mod set_once;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(registrar: &mut Registry) {
    let dir =  match TempDir::new("rustc_python_mixin") {
        Ok(d) => d,
        Err(e) => {
            registrar.sess.span_err(registrar.krate_span,
                                    &format!("could not create temporary directory: {}", e));
            return;
        }
    };
    registrar.register_syntax_extension(token::intern("python_mixin"),
                                        base::NormalTT(Box::new(Expander { dir: dir }),
                                                       None));
}

struct Expander {
    dir: TempDir
}

impl base::TTMacroExpander for Expander {
    fn expand<'cx>(&self,
                   cx: &'cx mut ExtCtxt,
                   sp: codemap::Span,
                   raw_tts: &[ast::TokenTree])
                   -> Box<MacResult+'cx>
    {
        macro_rules! mac_try {
            ($run: expr, $p: pat => $($fmt: tt)+) => {
                match $run {
                    Ok(x) => x,
                    Err($p) => {
                        cx.span_err(sp, &format!($($fmt)+));
                        return base::DummyResult::any(sp)
                    }
                }
            }
        }

        let (tts, option_tts): (_, &[_]) = match raw_tts.get(0) {
            Some(&ast::TtDelimited(_, ref delim)) if delim.delim == token::Brace => {
                (&raw_tts[1..], &delim.tts[..])
            }
            _ => (raw_tts, &[])
        };

        let opts = Options::parse(cx, option_tts);

        let code = match base::get_single_str_from_tts(cx, sp, tts, "`python_mixin!`") {
            Some(c) => c,
            None => return base::DummyResult::any(sp)
        };

        let lo = tts[0].get_span().lo;
        let first_line = cx.codemap().lookup_char_pos(lo).line as u64;

        let filename = cx.codemap().span_to_filename(sp);
        let path = PathBuf::new(&filename);
        let name = if path.is_absolute() {
            Path::new(path.file_name().unwrap())
        } else {
            &*path
        };
        let python_file = self.dir.path().join(name);

        mac_try! {
            fs::create_dir_all(python_file.parent().unwrap()),
            e => "`python_mixin!` could not create temporary directory: {}", e
        }

        let file = mac_try!(File::create(&python_file),
                            e => "`python_mixin!` could not create temporary file: {}", e);
        let mut file = io::BufWriter::new(file);

        mac_try!(io::copy(&mut io::repeat(b'\n').take(first_line - 1), &mut file),
                 e => "`python_mixin!` could not write output: {}", e);
        mac_try!(file.write(code.as_bytes()),
                 e => "`python_mixin!` could not write output: {}", e);
        mac_try!(file.flush(),
                 e => "`python_mixin!` could not flush output: {}", e);

        let command_name = format!("python{}", opts.version);
        let output = Command::new(&command_name)
            .current_dir(self.dir.path())
            .arg(name)
            .output();
        let output = mac_try!(output,
                              e => "`python_mixin!` could not execute `{}`: {}", command_name, e);

        if !output.status.success() {
            cx.span_err(sp,
                        &format!("`python_mixin!` did not execute successfully: {}", output.status));
            let msg = if output.stderr.is_empty() {
                "there was no output on stderr".to_string()
            } else {
                format!("the process emitted the following on stderr:\n{}",
                        String::from_utf8_lossy(&output.stderr))
            };
            cx.parse_sess().span_diagnostic.fileline_note(sp, &msg);
            return base::DummyResult::any(sp);
        } else if !output.stderr.is_empty() {
            cx.span_warn(sp, "`python_mixin!` ran successfully, but had output on stderr");
            let msg = format!("output:\n{}",
                              String::from_utf8_lossy(&output.stderr));
            cx.parse_sess().span_diagnostic.fileline_note(sp, &msg);
        }

        let emitted_code = mac_try!(String::from_utf8(output.stdout),
                                    e => "`python_mixin!` emitted invalid UTF-8: {}", e);

        let name = format!("<{}:{} python_mixin!>", filename, first_line);
        let parser = parse::new_parser_from_source_str(cx.parse_sess(),
                                                       cx.cfg(),
                                                       name,
                                                       emitted_code);

        Box::new(parser_any_macro::ParserAnyMacro::new(parser))
    }
}

const DEFAULT_VERSION: &'static str = "";

struct Options {
    version: String,
}

impl Options {
    fn parse(cx: &ExtCtxt, tts: &[ast::TokenTree]) -> Options {
        let mut version = set_once::SetOnce::new();
        let mut p = cx.new_parser_from_tts(tts);
        while p.token != token::Eof {
            // <name> = "..."
            let ident = p.parse_ident();
            let ident_span = p.last_span;
            match ident.as_str() {
                "version" => {
                    p.expect(&token::Eq);
                    let (s, _) = p.parse_str();
                    let span = codemap::mk_sp(ident_span.lo, p.last_span.hi);

                    if let Err(&(_, older_span)) = version.set((s.to_string(), span)) {
                        cx.span_err(span,
                                    &format!("`python_mixin!`: option `version` already set"));
                        cx.span_note(older_span,
                                     "set here");
                    }
                }
                s => {
                    cx.span_err(p.last_span, &format!("`python_mixing!`: unknown option `{}`",
                                                      s));
                    // heuristically skip forward, so we can check
                    // more things later.
                    while p.token != token::Comma {
                        p.bump();
                    }
                }
            }
            if p.token == token::Eof { break }
            p.expect(&token::Comma);
        }

        Options {
            version: version.get().map_or_else(|| DEFAULT_VERSION.to_string(), |t| t.0),
        }
    }
}
