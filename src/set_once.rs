pub struct SetOnce<T> {
    x: Option<T>
}
impl<T> SetOnce<T> {
    pub fn new() -> SetOnce<T> {
        SetOnce { x: None }
    }

    pub fn set(&mut self, x: T) {
        assert!(self.x.is_none());
        self.x = Some(x)
    }
    pub fn get(self) -> Option<T> {
        self.x
    }
}
