use super::Droplet;


impl<'a> Droplet<'a> {
    fn test_new(content: &'a[u8]) -> Self {
        Droplet {
            content
        }

    }
}


pub fn test_droplet<'a>(input_val : &'a[u8]) -> Droplet<'a> {
    Droplet::test_new(input_val.into())
}



