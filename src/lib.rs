use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Command)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Command for #name {
            fn base_offset(&self) -> usize {
                self.base_offset
            }

            fn set_byte_pair(&mut self, value: u8, offset: usize) -> Result<(), &'static str> {
                if offset < self.base_offset() {
                    return Err("Provided offset is less than the base offset");
                }

                if offset >= self.raw.len() - 1 {
                    return Err("Provided offset is greater than the length of the raw data");
                }

                self.raw[offset] = value;
                self.raw[offset + 0x1] = 0x55u8.wrapping_sub(value);

                Ok(())
            }

            fn id(&self) -> CommandId {
                self.raw[0x0].into()
            }

            fn set_id(&mut self, id: CommandId) {
                self.raw[0x0] = id as u8;
                self.set_checksum();
            }

            fn status(&self) -> u8 {
                self.raw[0x1]
            }

            fn set_status(&mut self, status: u8) {
                self.raw[0x1] = status;
                self.set_checksum();
            }

            fn eeprom_address(&self) -> EEPROMAddress {
                let addr = u16::from_le_bytes([self.raw[0x2], self.raw[0x3]]);
                addr.into()
            }

            fn set_eeprom_address(&mut self, address: EEPROMAddress) {
                self.raw[0x2..0x4].copy_from_slice(&(address as u16).to_le_bytes());
                self.set_checksum();
            }

            fn valid_data_len(&self) -> u8 {
                self.raw[0x4]
            }

            fn set_valid_data_len(&mut self, len: u8) {
                self.raw[0x4] = len;
                self.set_checksum();
            }

            fn checksum(&self) -> u8 {
                self.raw[0xf]
            }

            fn set_checksum(&mut self) {
                let sum: u8 = {
                    let sum_bytes: u16 = self.raw[0..0xf]
                        .iter()
                        .fold(0, |acc, &byte| acc + byte as u16);
                    ((self.report_id as u16 + sum_bytes) & 0xff) as u8
                };
                let checksum = 0x55u8.wrapping_sub(sum);
                self.raw[0xf] = checksum;
            }

            fn as_bytes(&self) -> &[u8] {
                self.raw.as_slice()
            }
        }
    };

    TokenStream::from(expanded)
}
