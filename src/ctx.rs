//! src/ctx.rs
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Ctx {
    user_id: usize,
    user_role: String,
}

impl Ctx {
    // Constructor
    #[allow(dead_code)]
    pub fn new(user_id: usize, user_role: String) -> Self {
        Self { user_id, user_role }
    }
    // Property Accessor
    #[allow(dead_code)]
    pub fn user_id(&self) -> usize {
        self.user_id
    }
    #[allow(dead_code)]
    pub fn user_role(&self) -> String {
        String::from(&self.user_role)
    }
}
