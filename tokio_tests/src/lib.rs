async fn double(n: u32) -> u32 {
    return n * 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn long_way_to_test_async() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert_eq!(rt.block_on(double(2)), 4);
    }

    #[tokio::test]
    async fn tokio_built_in_test() {
        assert_eq!(double(2).await, 4);
    }
}
