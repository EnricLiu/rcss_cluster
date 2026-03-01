pub mod v1;

pub trait Schema {
    fn verify(&self) -> Result<(), &'static str>;
}
