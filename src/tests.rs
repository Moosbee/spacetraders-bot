#[cfg(test)]
mod tests {

    #[test]
    fn exploration() {
        assert_eq!(4, 4);
    }

    #[test]
    #[should_panic]
    fn another() {
        panic!("Make this test fail");
    }
}
