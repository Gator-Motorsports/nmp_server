use tokio_util::codec::{Decoder, Encoder};

use crate::message::{self, Data, Message};

struct MessageCodec {}

impl MessageCodec {
    fn new() -> Self {
        Self {}
    }
}

fn get_length(item: &Message) -> u32 {
    let length = match item {
        Message::Signal(name, data) => {
            8 + name.len() as u32
                + 4 + match data {
                    message::Data::Integer(_) => 8,
                    message::Data::Float(_) => 8,
                    message::Data::Bool(_) => 1,
                }
        }
        Message::Subscription(name) => 8 + name.len() as u32,
    };

    length + 4
}

impl Encoder<Message> for MessageCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: Message,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.extend_from_slice(&get_length(&item).to_le_bytes());
        match item {
            Message::Signal(name, data) => {
                dst.extend_from_slice(&0u32.to_le_bytes());
                dst.extend_from_slice(&(name.len() as u64).to_le_bytes());
                for c in name.chars() {
                    dst.extend_from_slice(&(c as u8).to_le_bytes());
                }
                match data {
                    message::Data::Integer(v) => {
                        dst.extend_from_slice(&0u32.to_le_bytes());
                        dst.extend_from_slice(&v.to_le_bytes());
                    }
                    message::Data::Float(v) => {
                        dst.extend_from_slice(&1u32.to_le_bytes());
                        dst.extend_from_slice(&v.to_le_bytes());
                    }
                    message::Data::Bool(v) => {
                        dst.extend_from_slice(&2u32.to_le_bytes());
                        dst.extend_from_slice(&(v as u8).to_le_bytes());
                    }
                }
            }
            Message::Subscription(name) => {
                dst.extend_from_slice(&1u32.to_le_bytes());
                dst.extend_from_slice(&name.len().to_le_bytes());
                for c in name.chars() {
                    dst.extend_from_slice(&(c as u8).to_le_bytes());
                }
            }
        }

        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;

    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let length = u32::from_le_bytes(src[0..4].try_into().unwrap());
        let _ = src.split_to(4);

        if (src.len() as u32) < length {
            return Ok(None);
        }
        let mut bytes = src.split_to(length as usize);

        let variant = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let _ = bytes.split_to(4);

        match variant {
            0 => {
                let name_length = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
                let _ = bytes.split_to(8);
                let mut name = String::new();
                for i in 0..(name_length as usize) {
                    name.push(bytes[i] as char);
                }
                let _ = bytes.split_to(name_length as usize);

                let data_variant = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
                let _ = bytes.split_to(4);
                match data_variant {
                    0 => {
                        let number = i64::from_le_bytes(bytes[0..8].try_into().unwrap());
                        Ok(Some(Message::Signal(name, Data::Integer(number))))
                    }
                    1 => {
                        let number = f64::from_le_bytes(bytes[0..8].try_into().unwrap());
                        Ok(Some(Message::Signal(name, Data::Float(number))))
                    }
                    2 => {
                        let boolean = bytes[0] != 0;
                        Ok(Some(Message::Signal(name, Data::Bool(boolean))))
                    }
                    u => panic!("Unexpected data variant spotted {}", u),
                }
            }
            1 => {
                let name_length = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
                let _ = src.split_to(8);
                let mut name = String::new();
                for i in 0..(name_length as usize) {
                    name.push(bytes[i] as char);
                }

                Ok(Some(Message::Subscription(name)))
            }
            u => panic!("Unexpected variant spotted: {}", u),
        }
    }
}

#[cfg(test)]
mod test {
    use tokio_util::bytes::BytesMut;

    use super::*;

    #[test]
    fn test_encode_signal_integer() {
        let signal = Message::Signal("".to_string(), Data::Integer(11));

        let mut codec = MessageCodec::new();
        let mut bytes = BytesMut::new();

        codec.encode(signal.clone(), &mut bytes).unwrap();

        let back = codec.decode(&mut bytes).unwrap().unwrap();

        assert_eq!(signal, back);

    }

    #[test]
    fn test_encode_signal_float() {
        let signal = Message::Signal("".to_string(), Data::Float(11.0));

        let mut codec = MessageCodec::new();
        let mut bytes = BytesMut::new();

        codec.encode(signal.clone(), &mut bytes).unwrap();

        let back = codec.decode(&mut bytes).unwrap().unwrap();

        assert_eq!(signal, back);
    }

    #[test]
    fn test_encode_signal_bool() {
        let signal = Message::Signal("".to_string(), Data::Bool(false));

        let mut codec = MessageCodec::new();
        let mut bytes = BytesMut::new();

        codec.encode(signal.clone(), &mut bytes).unwrap();

        let back = codec.decode(&mut bytes).unwrap().unwrap();

        assert_eq!(signal, back);
    }
}
