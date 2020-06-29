#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let mut data = vec![0;100];
        data[1] = 1;
        let range = &data[0..10];
        println!("{:?}", range);
        let res: Vec<&u8> = range.iter().filter(|x| **x == 1).collect();
        println!("{:?}", res);
        println!("{:?}",res[0]);
    }
}