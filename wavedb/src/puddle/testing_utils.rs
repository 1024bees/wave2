use super::Droplet;


impl<'a> Droplet<'a> {
    fn test_new(content: &'a[u8]) -> Self {
        Droplet {
            content
        }

    }
}


pub fn test_droplet(input_val : &[u8]) -> Droplet<'_> {
    Droplet::test_new(input_val)
}



