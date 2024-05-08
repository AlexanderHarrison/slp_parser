// Adapted from picori 
// https://github.com/Julgodis/picori
//
//
// MIT License
// 
// Copyright (c) 2022 Julgodis
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod shift_jis_1997;

/// Adapted from picori: <https://github.com/Julgodis/picori>
///
/// Returns number of bytes written.
/// Stops decoding on null byte.
pub fn decode_shift_jis(bytes: &[u8], buffer: &mut String) -> Option<u32> {
    let mut byte_iter = bytes.iter().copied();
    let start_len = buffer.len() as u32;

    loop {
        let byte = match byte_iter.next() {
            Some(b) => b,
            None => break
        };

        if byte == 0x00 { break }

        // decode jis_x_0201 - single byte
        match byte {
            // Modified ASCII character
            0x5c => buffer.push('\u{00a5}'),
            0x7e => buffer.push('\u{203e}'),

            // Unaltered ASCII character
            0x00..=0x7f => buffer.push(byte as char),

            // Single-byte half-width katakana
            0xa1..=0xdf => {
                let unicode = 0xFF61 + (byte - 0xa1) as u32;
                buffer.push(char::from_u32(unicode)?);
            },

            // First byte of a double-width JIS X 0208 character
            0x81..=0x9F | 0xE0..=0xFC => {
                let byte_2 = byte_iter.next()?;

                let (first, last, offset) = shift_jis_1997::SJIS_1997_UTF8_T[byte as usize];

                if byte_2 < first || last < byte_2 { return None }

                let relative = (byte_2 - first) as usize;
                let index = offset as usize + relative;
                let value = shift_jis_1997::SJIS_1997_UTF8_S[index];
                if value == 0 { return None }

                // would prefer to not have to pull in this dependency,
                // but picori translates special characters into their fullwidth
                // variants, so I use decancer to prune that.
                // I could go through the generated tables to fix it but that is too much work.
                let options = decancer::Options::default().retain_japanese();
                match decancer::cure_char(unsafe { char::from_u32_unchecked(value as u32) }, options) {
                    decancer::Translation::Character(c) => buffer.push(c),
                    decancer::Translation::String(s) => buffer.push_str(s.as_ref()),
                    decancer::Translation::None => (),
                }
            }
            _ => return None,
        }
    }

    Some(buffer.len() as u32 - start_len)
}
