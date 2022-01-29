pub mod constants;
pub mod convert;
pub mod data;
pub mod decode;
pub mod derive;
pub mod mint;
pub mod snapshot;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
