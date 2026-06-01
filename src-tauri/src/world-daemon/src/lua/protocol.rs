use std::io::Read;
use flate2::read::ZlibDecoder;

const IAC: u8 = 255;
const SB: u8 = 250;
const SE: u8 = 240;
const WILL: u8 = 251;
const WONT: u8 = 252;
const DO: u8 = 253;
const DONT: u8 = 254;

const TELOPT_MCCP2: u8 = 86;
const TELOPT_GMCP: u8 = 201;

pub struct TelnetProtocolStateMachine {
    pub is_mccp_active: bool,
    pub has_sent_mccp: bool,
    pub has_sent_gmcp: bool,
}

impl TelnetProtocolStateMachine {
    pub fn new() -> Self {
        Self { 
            is_mccp_active: false,
            has_sent_mccp: false,
            has_sent_gmcp: false,
        }
    }

    pub fn handle_mccp_decompression(&mut self, compressed_bytes: &[u8]) -> Vec<u8> {
        if !self.is_mccp_active {
            return compressed_bytes.to_vec();
        }
        let mut decoder = ZlibDecoder::new(compressed_bytes);
        let mut decompressed = Vec::new();
        if decoder.read_to_end(&mut decompressed).is_ok() {
            decompressed
        } else {
            compressed_bytes.to_vec()
        }
    }

    pub fn parse_incoming_stream(
        &mut self,
        raw_bytes: &[u8],
        tcp_writer_tx: &mut Vec<u8>,
    ) -> (String, String, String) {
        let mut clean_bytes = Vec::with_capacity(raw_bytes.len());
        let mut gmcp_module = String::new();
        let mut gmcp_json = String::new();

        let mut i = 0;
        let n = raw_bytes.len();

        while i < n {
            if raw_bytes[i] == IAC && i + 2 < n {
                let command = raw_bytes[i + 1];
                let option = raw_bytes[i + 2];

                match command {
                    WILL => {
                        if option == TELOPT_MCCP2 {
                            if !self.has_sent_mccp {
                                tcp_writer_tx.extend_from_slice(&[IAC, DO, TELOPT_MCCP2]);
                                self.is_mccp_active = true;
                                self.has_sent_mccp = true;
                            }
                        } else if option == TELOPT_GMCP {
                            if !self.has_sent_gmcp {
                                tcp_writer_tx.extend_from_slice(&[IAC, DO, TELOPT_GMCP]);
                                self.has_sent_gmcp = true;
                            }
                        } else {
                            tcp_writer_tx.extend_from_slice(&[IAC, DONT, option]);
                        }
                        i += 3;
                    }
                    DO => {
                        if option == TELOPT_GMCP {
                            if !self.has_sent_gmcp {
                                tcp_writer_tx.extend_from_slice(&[IAC, WILL, TELOPT_GMCP]);
                                self.has_sent_gmcp = true;
                            }
                        } else {
                            tcp_writer_tx.extend_from_slice(&[IAC, WONT, option]);
                        }
                        i += 3;
                    }
                    SB => {
                        if option == TELOPT_GMCP {
                            let mut j = i + 3;
                            let mut sb_data = Vec::new();
                            while j < n {
                                if raw_bytes[j] == IAC && j + 1 < n && raw_bytes[j + 1] == SE {
                                    i = j + 2;
                                    break;
                                }
                                sb_data.push(raw_bytes[j]);
                                j += 1;
                            }
                            if let Ok(raw_str) = String::from_utf8(sb_data) {
                                let parts: Vec<&str> = raw_str.splitn(2, ' ').collect();
                                if parts.len() == 2 {
                                    gmcp_module = parts[0].trim().to_string();
                                    gmcp_json = parts[1].trim().to_string();
                                }
                            }
                        } else {
                            let mut j = i + 3;
                            while j < n {
                                if raw_bytes[j] == IAC && j + 1 < n && raw_bytes[j + 1] == SE {
                                    i = j + 2;
                                    break;
                                }
                                j += 1;
                            }
                        }
                    }
                    _ => { i += 3; }
                }
            } else {
                // 🟢【终极洗字】：过滤掉裸 255 和隐藏无法可视控制字，只放行合法的纯文本
                let b = raw_bytes[i];
                if b != IAC && b != 1 && b != 2 && b != 3 {
                    clean_bytes.push(b);
                }
                i += 1;
            }
        }

        // 自适应盲测解码
        let mut final_text = match std::str::from_utf8(&clean_bytes) {
            Ok(utf8_str) => utf8_str.to_string(),
            Err(_) => {
                let (decoded_gbk, _, _) = encoding_rs::GBK.decode(&clean_bytes);
                decoded_gbk.into_owned()
            }
        };

        if final_text.contains("<send") {
            final_text = final_text
                .replace("<send", "\x1b[1;36m[MXP超链接: ")
                .replace("\">", " ]\x1b[0m")
                .replace("</send>", "");
        }

        (final_text, gmcp_module, gmcp_json)
    }
}
