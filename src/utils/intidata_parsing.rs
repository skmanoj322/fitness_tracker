use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn parse_string(data: &str) -> (String, String) {
    let parse_data: Vec<(String, String)> = url::form_urlencoded::parse(data.as_bytes())
        .into_owned()
        .collect();

    let mut pairs: Vec<String> = Vec::new();
    let mut recived_hash = String::new();

    for (key, value) in parse_data {
        if key == "hash" {
            recived_hash = value;
        } else {
            pairs.push(format!("{}={}", key, value));
        }
    }

    pairs.sort();

    let data_check_string = pairs.join("\n");

    (data_check_string, recived_hash)
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_build_data_String() {
        let init_data = "query_id=AAHdF6IQAAAAAN0XohDhrOrc&user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%22%2C%22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C%22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%7D&auth_date=1662771648&hash=c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2";

        let (data_check_string, recived_hash) = parse_string(init_data);

        let expected_string = concat!(
            "auth_date=1662771648\n",
            "query_id=AAHdF6IQAAAAAN0XohDhrOrc\n",
            "user={\"id\":279058397,\"first_name\":\"Vladislav\",\"last_name\":\"Kibenko\",\"username\":\"vdkfrost\",\"language_code\":\"ru\",\"is_premium\":true}"
        );

        println!("{:?}", expected_string);

        let expected_hash = "c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2";

        assert_eq!(data_check_string, expected_string);
        assert_eq!(recived_hash, expected_hash);
    }
}

pub fn create_secret_key(bot_token: &str) -> Vec<u8> {
    let mut hmca_token: Hmac<Sha256> =
        Hmac::new_from_slice(b"WebAppData").expect("Hmac can take key of any size");

    hmca_token.update(bot_token.as_bytes());

    let secret_key = hmca_token.finalize();

    secret_key.into_bytes().to_vec()
}

pub fn verify_signature(secret_key: &[u8], data_check_string: &str, recevied_hash: &str) -> bool {
    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(secret_key).expect("Hmca can take of any size");

    mac.update(data_check_string.as_bytes());

    // let computed_hash = hex::encode(result_bytes);

    let recevied_bytes = match hex::decode(recevied_hash) {
        Ok(b) => b,
        Err(_) => return false,
    };

    mac.verify_slice(&recevied_bytes).is_ok()
}
