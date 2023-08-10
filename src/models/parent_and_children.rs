use serde::Serialize;

#[derive(Serialize)]
pub struct ParentAndChildren<'a, T, U>
where
    T: Serialize,
    U: Serialize,
{
    pub parent: T,
    pub children: &'a [U],
}
