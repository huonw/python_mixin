pub struct SetOnce<T> {
    x: Option<T>
}
impl<T> SetOnce<T> {
    pub fn new() -> SetOnce<T> {
        SetOnce { x: None }
    }

    pub fn set(&mut self, x: T) -> Result<(), &T> {
        if self.x.is_none() {
            self.x = Some(x);
            Ok(())
        } else {
            Err(self.x.as_ref().unwrap())
        }
    }
    pub fn get(self) -> Option<T> {
        self.x
    }
}
