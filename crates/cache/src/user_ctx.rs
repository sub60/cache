use types::UserName;

#[derive(Clone)]
#[expect(dead_code)]
pub(crate) struct UserCtx {
    pub(crate) name: UserName,
}
