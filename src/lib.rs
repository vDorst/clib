use std::{ffi::CStr, usize};

pub fn port(data: [u8; 2]) -> u8 {
    let mut port = data[0] & 0xc0;
    port |= data[1] & 0x03;
    port = (port << 2) | (port >> 6);
    //port = port.rotate_left(2);

    // let mut port = (data[0] >> 6) & 0x3;
    // port |= (data[1] & 0x03) << 2;

    port
}

pub fn atoi_hex(mut idx: usize, cmd_buffer: &[u8], hex_value: &mut [u8; 4]) -> usize {
    let mut h_idx = 0;
    let mut val = 0_u8;
    let mut c;

    loop {
        c = cmd_buffer[idx];

        println!("c {} val {val}, idx {idx} {h_idx}", char::from(c));

        if c == b' ' || c == b'\0' {
            break;
        }

        // swap hex nibbles
        val = (val >> 4) | (val << 4);

        if c - b'0' < 10 {
            println!("\t0-9");
            val |= c - b'0';
        } else {
            c |= 0x20;
            // b'E';
            // b'e';
            println!("\t{}", char::from(c));
            c = c.wrapping_sub(b'a');
            // println!("c {c}");
            if c > 5 {
                println!("other");
                h_idx = 0;
                break;
            }
            println!("\tA-F");
            val |= c + 10;
        }

        idx += 1;
        hex_value[h_idx >> 1] = val;

        println!("\t- VAL: {val:02x}");

        if h_idx & 1 == 1 {
            val = 0;
        }
        h_idx += 1;
    }
    println!();

    if h_idx & 1 == 1 {
        println!("\tOdd hex value");
        hex_value[h_idx >> 1] <<= 4;
        hex_value[3] = hex_value[3] >> 4 | (hex_value[2] << 4);
        hex_value[2] = hex_value[2] >> 4 | (hex_value[1] << 4);
        hex_value[1] = hex_value[1] >> 4 | (hex_value[0] << 4);
        hex_value[0] >>= 4;
    }

    return (h_idx + 1) >> 1;
}

pub fn sfr_data_to_html(sfr_data: [u8; 4], out: &mut Vec<u8>) {
    let mut print_zeros = 0;
    let mut val = 0;

    for nibble in 0..8 {
        if (nibble & 1) == 0 {
            val = sfr_data[nibble >> 1];
        }
        // force the swap instruction.
        val = (val << 4) | (val >> 4);
        // when print_zeros is not zero, we know that a non-zero number has printed.
        // That have to print all the next numbers.
        print_zeros |= val;
        // only care about lower nibble, that is what is printed.
        print_zeros &= 0x0f;
        if print_zeros != 0 {
            out.push(itohex(val));
        }
    }
    if print_zeros == 0 {
        out.push(b'0');
    }
}

pub fn gpio_pin_test(pin: u8, regdata: u64) -> u8 {
    let reg_data = regdata.to_le_bytes();
    for (idx, reg) in reg_data.chunks_exact(4).enumerate() {
        println!("reg[{idx}]: {reg:02x?}");
    }

    let reg = if pin > 31 { 4 } else { 0 };

    let idx = (usize::from(pin) >> 3) & 3;
    let bit = 1 << (pin & 7);

    let ret = reg_data[reg + idx] & bit;

    println!("R{reg}, I{idx} B{bit:02x} = {ret:02x}");

    ret
}

pub fn itohex(mut val: u8) -> u8 {
    val &= 0x0f;
    val = val.wrapping_sub(10);
    if val.cast_signed() >= 0 {
        val = val.wrapping_add(b'a' - b'0' - 10);
    }
    val = val.wrapping_add(b'0' + 10);

    val
}

pub fn itoa_html(v: u8, out: &mut Vec<u8>) {
    println!("Input: {v}");
    let mut t: u8 = v / 100;
    let mut print_zero: u8 = t;
    if print_zero > 0 {
        println!("100: {t}");
        out.push(t.wrapping_add(b'0'));
    }
    t = (v / 10) % 10;
    print_zero |= t;
    if print_zero > 0 {
        println!("10: {t}");
        out.push(t.wrapping_add(b'0'));
    }
    t = v % 10;
    println!("1: {t}");
    out.push(t.wrapping_add(b'0'));
}

pub fn byte_to_html(mut val: u8, out: &mut Vec<u8>) {
    let mut again: u8 = 2;
    loop {
        val = val.rotate_left(4);
        out.push(itohex(val));
        again -= 1;
        if again == 0 {
            break;
        }
    }
}

pub fn isletter(mut l: u8) -> bool {
    // return (l >= 'a' && l <= 'z') || (l >= 'A' && l <= 'Z');

    // Make it lowercase
    l |= 0x20;
    l = l.wrapping_sub(b'a');
    l <= (b'z' - b'a')
}

pub fn parse_short(cmd_buffer: &[u8], data: &mut u16) -> bool {
    *data = 0;
    let mut err = true;

    for b in cmd_buffer {
        let c = (*b).wrapping_sub(b'0');
        if c > 9 {
            break;
        }
        err = false;
        *data = *data * 10 + u16::from(c);
    }

    err
}

pub fn parse_i16(val_inp: u16) -> i16 {
    // let mut val = (val_inp & 0x7FFF) as i16;
    // if val_inp & 0x8000 != 0x00 {
    //     val = val.wrapping_add(-1);
    // }

    val_inp.cast_signed()
}

pub struct Flash<'data> {
    buf: &'data [u8],
    addr: usize,
}

impl<'data> Flash<'data> {
    #[expect(dead_code)]
    fn set_addr(&mut self, addr: u32) {
        self.addr = addr as usize;
    }

    #[track_caller]
    fn exec_go(&mut self) {
        assert!(
            self.buf.get(self.addr..self.addr + 4).is_some(),
            "Can't grab 4 bytes at addr {:04x?}",
            self.addr
        );
    }

    fn sfr_flash_data0(&self) -> u8 {
        self.buf.get(self.addr).copied().unwrap()
    }
    fn sfr_flash_data8(&self) -> u8 {
        self.buf.get(self.addr + 1).copied().unwrap()
    }
    fn sfr_flash_data16(&self) -> u8 {
        self.buf.get(self.addr + 2).copied().unwrap()
    }
    fn sfr_flash_data24(&self) -> u8 {
        self.buf.get(self.addr + 3).copied().unwrap()
    }

    fn inc_addr(&mut self, arg: u32) {
        self.addr += arg as usize;
    }
}

#[track_caller]
pub fn flash_find_mark(flash: &mut Flash<'_>, mark: &[u8], mut len: u16) -> u16 {
    let mut mpos = 0;
    let mut fifobuf = [0x00; 16];

    let mut fifo_i = 0;
    let mut search_ptr_markbuf_k = 0;

    // Calculate the length
    while mark[fifo_i] != 0 {
        fifo_i += 1;
    }
    let mark_len_l = fifo_i;

    if mark_len_l >= 12 {
        mpos = 0xffff;
        return mpos;
    }

    fifo_i = 0;
    let mut cmp_ptr_mark_j = 0;

    loop {
        flash.exec_go();

        fifobuf[fifo_i] = flash.sfr_flash_data0();
        fifo_i += 1;
        fifobuf[fifo_i] = if len >= 1 { flash.sfr_flash_data8() } else { 0 };
        fifo_i += 1;
        fifobuf[fifo_i] = if len >= 2 {
            flash.sfr_flash_data16()
        } else {
            0
        };
        fifo_i += 1;
        fifobuf[fifo_i] = if len >= 3 {
            flash.sfr_flash_data24()
        } else {
            0
        };
        fifo_i += 1;

        // println!("MARKBUF = {fifobuf:02x?}");
        fifo_i &= 0xf;

        flash.inc_addr(4);

        while mark[cmp_ptr_mark_j] != 0 && (search_ptr_markbuf_k != fifo_i) {
            if mark[cmp_ptr_mark_j] != fifobuf[search_ptr_markbuf_k] {
                // not match
                search_ptr_markbuf_k = search_ptr_markbuf_k - cmp_ptr_mark_j;
                cmp_ptr_mark_j = usize::MAX;
            }
            cmp_ptr_mark_j = cmp_ptr_mark_j.wrapping_add(1);
            search_ptr_markbuf_k = (search_ptr_markbuf_k + 1) & 0xf;
            // println!("\tK = {search_ptr_markbuf_k}");
        }
        if mark[cmp_ptr_mark_j] == 0 {
            // println!(
            //     "End Search K={search_ptr_markbuf_k} {}, mark_len_l {mark_len_l}, mpos {mpos}",
            //     search_ptr_markbuf_k & 0x03
            // );
            mpos += ((fifo_i + search_ptr_markbuf_k) as u16) & 0x3;
            return mpos;
        }

        mpos += 4;

        if len <= 4 {
            break;
        }
        len -= 4;
    }
    mpos = 0xffff;
    mpos
}

#[derive(Debug, PartialEq)]
pub enum ERR {
    Ok,
    TooManyArgs,
    CmdTooLong,
}

const CMD_BUF_SIZE: u8 = 128;
const N_WORDS: u8 = 16;

fn cmd_tokenize(
    cmd_buffer: &[u8; CMD_BUF_SIZE as usize],
    cmd_words_b: &mut [i8; N_WORDS as usize],
    err_status: &mut ERR,
) -> u8 {
    *err_status = ERR::Ok;
    let mut line_ptr: u8 = 0;
    let mut is_white: bool = true;
    let mut word: u8 = 0;

    let mut c;

    cmd_words_b.iter_mut().for_each(|val| *val = -1);

    loop {
        c = cmd_buffer[line_ptr as usize];
        if c == b'\0' {
            cmd_words_b[word as usize] = line_ptr as i8;
            return 0;
        }

        if line_ptr >= CMD_BUF_SIZE - 1 {
            *err_status = ERR::CmdTooLong;
            return 1;
        }

        if is_white && c != b' ' {
            is_white = false;
            cmd_words_b[word as usize] = line_ptr as i8;
            word += 1;
        } else if c == b' ' {
            is_white = true;
        }
        line_ptr += 1;
    }
}

fn cmd_compare(
    start: u8,
    text: &CStr,
    cmd_buffer: &[u8; CMD_BUF_SIZE as usize],
    cmd_words_b: &[i8; N_WORDS as usize],
) -> u8 {
    let cmd = text.to_bytes_with_nul();

    if (start > 0) && (cmd_words_b[start as usize] <= 0) {
        // nothing on this word -> no match
        return 0;
    }

    let mut j = 0_i8;
    let mut i = cmd_words_b[start as usize];
    while i != cmd_words_b[start as usize + 1] {
        if cmd_buffer[i as usize] == b' ' {
            break;
        }
        i &= (CMD_BUF_SIZE - 1).cast_signed();
        if cmd[j as usize] == 0 {
            // end of command reached, but cmd_buffer has more characters, so no match
            return 0;
        }
        let c = cmd[j as usize];
        j += 1;
        if cmd_buffer[i as usize] != c {
            break;
        }
        i += 1;
    }
    // check next word reached and command fully matched
    if ((i == cmd_words_b[start as usize + 1]) || (cmd_buffer[i as usize] == b' '))
        && cmd[j as usize] == 0
    {
        return 1;
    }
    0
}

fn execute_config(buf: &[u8]) -> ERR {
    let mut err_status = ERR::Ok;
    let mut flash_reader = buf.chunks_exact(256);
    let mut cmd_buffer: [u8; CMD_BUF_SIZE as usize] = [0; _];
    let mut cmd_words_b: [i8; 16] = [0; _];

    let mut cmd_idx: u8 = 0;
    'lus: loop {
        let Some(flashbuf) = flash_reader.next() else {
            break;
        };
        let mut cfg_idx: u8 = 0;
        let mut c;

        loop {
            if cmd_idx >= (CMD_BUF_SIZE - 1) {
                cmd_buffer[usize::from(cmd_idx)] = b'\0';
                println!("ERROR: Command too long: {cmd_buffer:02x?}");
                err_status = ERR::CmdTooLong;
                break 'lus;
            }

            c = flashbuf[usize::from(cmd_idx)];
            println!("C {c:02x} cmd_idx: {cmd_idx}");
            cfg_idx = cfg_idx.wrapping_add(1);

            if c == 0 || c == b'\n' {
                cmd_buffer[usize::from(cmd_idx)] = b'\0';
                // if cmd_idx != 0 || cmd_tokenize(&cmd_buffer, &mut cmd_words_b, &mut err_status) == 0
                // {
                // }

                if c == 0 {
                    break 'lus;
                }
            }

            cmd_buffer[usize::from(cmd_idx)] = c;
            cmd_idx += 1;
            if cfg_idx == 0 {
                break;
            }
        }
    }

    err_status
}

#[cfg(test)]
mod tests_flash {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn test_config_read() {
        let mut cmd_buffer = [0; 512];
        assert_eq!(execute_config(&cmd_buffer), ERR::Ok);

        for idx in &mut cmd_buffer[0..126] {
            *idx = b'b';
        }
        assert_eq!(execute_config(&cmd_buffer), ERR::Ok);

        for idx in &mut cmd_buffer[0..127] {
            *idx = b'b';
        }
        assert_eq!(execute_config(&cmd_buffer), ERR::CmdTooLong);

        const BAD_CONFIG: &CStr = c"ip 192.168.10.247
gw 192.168.10.1
netmask 255.255.255.0
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
eee status";
        cmd_buffer[0..BAD_CONFIG.to_bytes_with_nul().len()]
            .copy_from_slice(BAD_CONFIG.to_bytes_with_nul());
        assert_eq!(execute_config(&cmd_buffer), ERR::CmdTooLong);
    }

    #[test]
    fn cmd_compare_test() {
        const WD: i8 = -1;
        let mut cmd_buffer = [0; CMD_BUF_SIZE as usize];
        let mut word_buf = [WD; N_WORDS as usize];

        // Empty buffer
        assert_eq!(cmd_compare(0, c"test", &cmd_buffer, &word_buf), 0);

        const GOOD_CONFIG_1: &CStr = c"ip 192.168.10.247";
        cmd_buffer[0..GOOD_CONFIG_1.to_bytes_with_nul().len()]
            .copy_from_slice(GOOD_CONFIG_1.to_bytes_with_nul());

        word_buf = [0, 3, 17, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD];
        assert_eq!(cmd_compare(0, c"ip", &cmd_buffer, &word_buf), 1);
        assert_eq!(cmd_compare(1, c"192.168.10.247", &cmd_buffer, &word_buf), 1);
        assert_eq!(cmd_compare(1, c"192.168.10.24", &cmd_buffer, &word_buf), 0);
    }

    #[test]
    fn cmd_tokenize_test() {
        const WD: i8 = -1;
        let mut cmd_buffer = [0; CMD_BUF_SIZE as usize];
        let mut word_buf = [WD; N_WORDS as usize];
        let mut err_status: ERR = ERR::Ok;

        // Empty buffer
        assert_eq!(cmd_tokenize(&cmd_buffer, &mut word_buf, &mut err_status), 0);
        assert_eq!(err_status, ERR::Ok);
        assert_eq!(
            word_buf,
            [
                0, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD
            ]
        );

        // Corrupted, only spaces
        cmd_buffer.iter_mut().for_each(|val| *val = b' ');
        assert_eq!(cmd_tokenize(&cmd_buffer, &mut word_buf, &mut err_status), 1);
        assert_eq!(err_status, ERR::CmdTooLong);
        assert_eq!(
            word_buf,
            [
                WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD
            ]
        );

        // Valid command
        const GOOD_CONFIG_1: &CStr = c"ip 192.168.10.247";
        cmd_buffer[0..GOOD_CONFIG_1.to_bytes_with_nul().len()]
            .copy_from_slice(GOOD_CONFIG_1.to_bytes_with_nul());

        assert_eq!(cmd_tokenize(&cmd_buffer, &mut word_buf, &mut err_status), 0);
        assert_eq!(err_status, ERR::Ok);
        assert_eq!(
            word_buf,
            [0, 3, 17, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD]
        );

        // Valid command
        const GOOD_CONFIG_2: &CStr = c"vlan 1 2t";
        cmd_buffer[0..GOOD_CONFIG_2.to_bytes_with_nul().len()]
            .copy_from_slice(GOOD_CONFIG_2.to_bytes_with_nul());

        assert_eq!(cmd_tokenize(&cmd_buffer, &mut word_buf, &mut err_status), 0);
        assert_eq!(err_status, ERR::Ok);
        assert_eq!(
            word_buf,
            [0, 5, 7, 9, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD]
        );

        // Valid command
        const GOOD_CONFIG_3: &CStr = c"port";
        cmd_buffer[0..GOOD_CONFIG_3.to_bytes_with_nul().len()]
            .copy_from_slice(GOOD_CONFIG_3.to_bytes_with_nul());

        assert_eq!(cmd_tokenize(&cmd_buffer, &mut word_buf, &mut err_status), 0);
        assert_eq!(err_status, ERR::Ok);
        assert_eq!(
            word_buf,
            [0, 4, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD, WD]
        );
    }

    #[test]
    #[ignore = "broken"]
    fn it_works() {
        let mut f = Flash {
            buf: &[0; 256],
            addr: 0x000,
        };
        assert_eq!(
            flash_find_mark(&mut f, c"#{".to_bytes_with_nul(), 256),
            0xFFFF
        );

        let mut f = Flash {
            //     01234567890123456789
            buf: b"14309231j#{asdasdadads",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"#{".to_bytes_with_nul(), 22), 11);

        let mut f = Flash {
            //     01234567890123456789
            buf: b"#{asdasdadads14309231j",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"#{".to_bytes_with_nul(), 22), 2);

        let mut f = Flash {
            //     01234567890123456789
            buf: b"a#{asdasdadads14309231j",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"#{".to_bytes_with_nul(), 22), 3);

        let mut f = Flash {
            //     01234567890123456789
            buf: b"#{asdasdadads14309231j",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"#{a".to_bytes_with_nul(), 22), 3);

        let mut f = Flash {
            //     0123456789012345678901234567890123456789
            buf: b"asdasdadads1430923#{aaaa",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"#{a".to_bytes_with_nul(), 24), 21);

        let mut f = Flash {
            //     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890
            buf: b"asdasdadads14309aaaaaaaaaaaaaaaaaaaaaaaaaasdsadsad3e3ddfadasdaedsdf3rwafef21fwgfwa23#{aaaa",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"#{a".to_bytes_with_nul(), 90), 87);

        let mut f = Flash {
            //     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890
            buf: &[0; 0x1000],
            addr: 0x000,
        };
        assert_eq!(
            flash_find_mark(&mut f, c"#{a".to_bytes_with_nul(), 90),
            0xFFFF
        );

        let mut f = Flash {
            //     0123456789012345678901234567890123456789
            buf: b"ip 192.1\ngw 123018391\ngw 123018391\n",
            addr: 0x000,
        };
        assert_eq!(flash_find_mark(&mut f, c"\n".to_bytes_with_nul(), 24), 9);
        assert_eq!(flash_find_mark(&mut f, c"\n".to_bytes_with_nul(), 24), 13);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_i16_test() {
        assert_eq!(parse_i16(0x7FFF), 0x7FFF);
        assert_eq!(parse_i16((-1_i16).cast_unsigned()), -1);
        assert_eq!(parse_i16(0x8000), i16::MIN);
    }

    #[test]
    fn isletter_test() {
        assert!(isletter(b'a'));
        assert!(isletter(b'A'));
        assert!(isletter(b'Z'));
        assert!(isletter(b'z'));

        assert!(!isletter(b'0'));
        assert!(!isletter(b'?'));
        assert!(!isletter(b'.'));
    }

    #[test]
    fn to_hex() {
        assert_eq!(itohex(0), b'0');
        assert_eq!(itohex(1), b'1');
        assert_eq!(itohex(9), b'9');
        assert_eq!(itohex(10), b'a');
        assert_eq!(itohex(15), b'f');
        assert_eq!(itohex(16), b'0');
    }

    #[test]
    fn atio_hex_test() {
        let mut hex_value = [0; 4];
        assert_eq!(atoi_hex(0, b"1\0", &mut hex_value), 1);
        assert_eq!(hex_value[0], 1);

        assert_eq!(atoi_hex(0, b"12\0", &mut hex_value), 1);
        assert_eq!(hex_value[0], 0x12);

        assert_eq!(atoi_hex(0, b"FF\0", &mut hex_value), 1);
        assert_eq!(hex_value[0], 0xFF);

        assert_eq!(atoi_hex(1, b"FF\0", &mut hex_value), 1);
        assert_eq!(hex_value[0], 0xF);

        assert_eq!(atoi_hex(0, b"AeFba3\0", &mut hex_value), 3);
        assert_eq!(hex_value[0], 0xAE);
        assert_eq!(hex_value[1], 0xFB);
        assert_eq!(hex_value[2], 0xA3);

        assert_eq!(atoi_hex(1, b"AeFba3\0", &mut hex_value), 3);
        assert_eq!(hex_value[0], 0x0e);
        assert_eq!(hex_value[1], 0xfb);
        assert_eq!(hex_value[2], 0xA3);

        assert_eq!(atoi_hex(0, b"AABBC\0", &mut hex_value), 3);
        assert_eq!(hex_value[0], 0x0A);
        assert_eq!(hex_value[1], 0xAB);
        assert_eq!(hex_value[2], 0xBC);
        assert_eq!(hex_value[3], 0x0);

        assert_eq!(atoi_hex(1, b"AqeFba\0", &mut hex_value), 0);
    }

    #[test]
    fn hextohtml_test() {
        let mut out = vec![];

        out.clear();
        byte_to_html(0, &mut out);
        assert_eq!(&out, b"00");

        out.clear();
        byte_to_html(1, &mut out);
        assert_eq!(&out, b"01");

        out.clear();
        byte_to_html(10, &mut out);
        assert_eq!(&out, b"0a");

        out.clear();
        byte_to_html(99, &mut out);
        assert_eq!(&out, b"63");

        out.clear();
        byte_to_html(100, &mut out);
        assert_eq!(&out, b"64");

        out.clear();
        byte_to_html(255, &mut out);
        assert_eq!(&out, b"ff");
    }

    #[test]
    fn itohtml_test() {
        let mut out = vec![];

        out.clear();
        itoa_html(0, &mut out);
        assert_eq!(&out, b"0");

        out.clear();
        itoa_html(1, &mut out);
        assert_eq!(&out, b"1");

        out.clear();
        itoa_html(10, &mut out);
        assert_eq!(&out, b"10");

        out.clear();
        itoa_html(100, &mut out);
        assert_eq!(&out, b"100");

        out.clear();
        itoa_html(255, &mut out);
        assert_eq!(&out, b"255");
    }

    #[test]
    fn sfr_print_test() {
        let mut out = vec![];

        out.clear();
        sfr_data_to_html(0_u32.to_be_bytes(), &mut out);
        assert_eq!(&out, b"0");

        out.clear();
        sfr_data_to_html([0x12, 0xAA, 0xBB, 0x78], &mut out);
        assert_eq!(&out, b"12aabb78");

        out.clear();
        sfr_data_to_html([0x02, 0xAA, 0xBB, 0x78], &mut out);
        assert_eq!(&out, b"2aabb78");

        out.clear();
        sfr_data_to_html(0xF_u32.to_be_bytes(), &mut out);
        assert_eq!(&out, b"f");

        out.clear();
        sfr_data_to_html(0x12_u32.to_be_bytes(), &mut out);
        assert_eq!(&out, b"12");

        out.clear();
        sfr_data_to_html(0x123_u32.to_be_bytes(), &mut out);
        assert_eq!(&out, b"123");

        out.clear();
        sfr_data_to_html(0x12345678_u32.to_be_bytes(), &mut out);
        assert_eq!(&out, b"12345678");

        out.clear();
        sfr_data_to_html(u32::MAX.to_be_bytes(), &mut out);
        assert_eq!(&out, b"ffffffff");
    }

    #[test]
    fn short_test() {
        let mut data = 0;
        assert!(!parse_short(b"112", &mut data));
        assert_eq!(data, 112);

        assert!(parse_short(b"", &mut data));
        assert_eq!(data, 0);

        assert!(!parse_short(b"12345", &mut data));
        assert_eq!(data, 12345);

        assert!(!parse_short(b"123a45", &mut data));
        assert_eq!(data, 123);
    }

    #[test]
    #[ignore = "broken"]
    fn gpio_read_test() {
        let data: u64 = 1 << 30;
        assert_eq!(gpio_pin_test(30, data), 0x80);
    }

    #[test]
    fn port_test() {
        for port_num in 0_u8..16 {
            let data: [u8; 2] = [
                (port_num & 0x3).rotate_right(2) | 0x3f,
                ((port_num >> 2) & 0x3) | 0xFC,
            ];
            println!("port: {port_num}, data: {data:02x?}");
            assert_eq!(port(data), port_num);
        }
    }
}
