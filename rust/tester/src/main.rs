use aes_gcm::{aead::{Aead, KeyInit, Payload}, Aes256Gcm, Nonce, Key}; 
use hex::{decode, encode}; // Use hex for decoding and encoding
use anyhow::{Result, Context}; // Use anyhow for error handling
use std::str;
use base64::{encode as base64_encode}; 

fn decrypt_data(
    key_hex: &str,
    iv_hex: &str,
    raw_ciphertext_hex: &str, // Raw ciphertext + AAD + Tag (as hex)
    aad_length: usize,        // Length of AAD in bytes
) -> Result<String> {

    let raw_ciphertext = decode(raw_ciphertext_hex).context("Failed to decode raw ciphertext")?;
    let aad = &raw_ciphertext[..aad_length];
    let ciphertext = &raw_ciphertext[aad_length..raw_ciphertext.len()];

    println!("Nonce (hex): {}", encode(&aad));
    println!("Nonce (hex): {}", encode(&ciphertext));
    
    // Wrap ciphertext|tag and AAD in Payload struct
    let payload = Payload {
        msg: &ciphertext[..],
        aad: &aad[..],
    };

    // Sequence number maybe is wrong
    let sequence_number = 0u64;

    // Hex decode key and iv, import both
    let iv = decode(&iv_hex).context("Failed to decode iv")?;
    let key = decode(&key_hex).context("Failed to decode key")?;

    let mut nonce = vec![0u8; 12]; // 12-byte nonce
    let seq_num_bytes = sequence_number.to_be_bytes(); // Convert sequence number to bytes

    // Copy the sequence number into the last 8 bytes of the nonce
    nonce[4..12].copy_from_slice(&seq_num_bytes); // Place the sequence number into the last 8 bytes of the nonce

    // XOR the last 8 bytes of the IV with the sequence number
    for i in 0..12 {
        nonce[i] ^= iv[i]; // XOR IV with the sequence number bytes
    }

    // Create Nonce
    let nonce = Nonce::from_slice(&nonce); 
    let key = Key::<Aes256Gcm>::from_slice(&key[..]);

    // Decrypt
    let cipher = Aes256Gcm::new(&key);
    let decrypted = cipher.decrypt(&nonce, payload).map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    // Return decrypted data as a hexadecimal string
    if let Ok(readable_text) = std::str::from_utf8(&decrypted) {
        Ok(readable_text.to_string())
    } else {
        Ok(hex::encode(&decrypted))
    }
}

fn main() {
    // Input data (hex-encoded)
    let rx_key_hex = "c8af3ba57ceee8b4c6bfa02dec92c626463176ab6973a6a0faba35fd7092cadc";
    let rx_iv_hex = "c37220408c5f57f715c70345"; 
    let raw_rx_ciphertext_hex_0 = "17030300fadd9c209edeaa70a1559031916bcfb7a0d2fa9df276eb63d65aa83ef1789c4e937e2cfddf353647dc2f79f2580109b6100b89a05f593777e4d80a220c4a05fa6aa713b8294e136b140daab25b2481eecc9519fab26f63a39822a3c814666c462194efc24f8163e16900eee47afb4d9aff40a55b1e7465a5b50d22dc14fc96ef42d4f5056c6d8d22796e8a45c0eebff171036aa93d17883f13b87dc750fb7aca589d7513bd859575baa5ac7f8a25560658e44e10cfa3e7098654ac843bbdac8a5c9981037bb94a0235529e017edef24748e81b0f9467fa59728f12d5e85624082ab6ed441042d6cddd9fd15de83445c789a64bbfaa6b7c5f3fb4e4";
    let raw_rx_ciphertext_hex_1 = "17030300fac49b521379ab578bb9ca5a9d25f03bd8f15348094c7a0a02a033ffba19ce939b820d7207bd34a021daeb975a8e23ed53087ca1f57d769eb3fe17bc545c3b08d74ad4fdd7822bbe877d5d31551b7eff2e39d1ce876aad9edc5192707428aa49b855e31312ceb89b1c81fb06fb46aac349fb8b02e40391c38697e1821d27ad36a1b5ac6d43e469d8f057f54778e0b8382f501a1662a2d8c532e8bf36e4995a451802b7f14539643ff53d952ada403fa3c1634df9231378755847143b9eb687c7312e39447eb01534310d10c89cb01d4f707b2d75d7fbe6e67ea14c755b7c326c22ec86aafc8f3b4482e131d9eb832b70409fa32dd329c660234663";
    let raw_rx_ciphertext_hex_2 = "1703030175dabc84bcd8aba5d26f85881f356f2168be251f14915f33015009f9c121d3d77829f415484d62d6045dc974e70129581bdea7e06764550b6d43233244c021884f79a981e116f6581e7d231f70a53c2bde49b1d4cdda7578ba6b7003563c90f22f7d0e2cbdea7c39e61c205b507a6d32b93ae95596a449bff67bfeef092351ecff605ee1d402fda4e03e1b76a8ce471542d35f40e963504424afa2d98775672d181688a5b63905bdd165dd759db3db664422223e645357bcaa689fe07779471f8e00cf454aba6c46c627ca6137e19bef1159ee903c6225b258db736e6480b1a0e4a7bfb502a999576e2353708030c90cef2179999ccba0393c3dadc94966b41231c81895bfc9afe9c46c20cc0c9e37722cb86b7eee7d7022a259d9750b6d2ec497d303795bea9dd0b8aaae17f56d7cee5d704dc56ed653131f100ad49c97f8852e1edb52c035fb6dc59515ee8a42de603e172ab8eb56d69b24da4d30a93992220665828a92cdd9ddae281fe3978ede57a52d5f97bccf";
    let raw_rx_ciphertext_hex_3 = "17030304f9459442c83920645394115ff8d5ffeebf56719013a78bfc0a08513b30a35062ba687d69b2795edf1e3d1328b6b10775fc2fa36dfcfe7bc4995f753ef9ab39009c690ae872e86313cbd133203a3486556ca92dfbcf7f9ae379a0407187b5720295c05faa43a795dad2c22ebb77841a076efb330f1a40a01f8e197670f6f24059d88c7036354ff6c7713f2fc8246d048e6e7e10561aa0f4618634f16a5c43ea3b036295ebfee44a7894c96bf2dd96f23948bd87cc6b815b9715881391a69ba76964882b35d43a52b2e979265d28bb5f610f6b346a01e8f367bf7b3d0caababcbffde13d0bed5c667c4f51d7ee1a4864545ac3d92b41404aeaeff89ce1af7bc848bf00b224a2b5a443204895dadcc3bdde246e357012a9a0f72fced996b85a74951982b81f21b8f78b2e2e91efc0139b850830b4e364a4a2da1b269f7839c62afa2f982be0af4232328fc87eb5767e8212c18a9c10fa9b4323fbf61dace09341d86583642bd57476af9abadca23b2b7002a6b6312ed3a030a8c1c77de34fbe2e78e6eaab99fc204e8d36a04e1647dab7f911316fea278104dc39d229b2cb952439bbeb857bf66575150e18c1b0a064fead8b92783b210b8646e39a7576f87af8caf8376808805217d682bcac6543a7a6d5d7ee391bea8ba834109333dca9a13ec0cdcb7604e2cf9a73273b953f11697797a2c731c94f4960c8ed6cbd12ff244a7c83c7f559069ac1c95f6ee7cb612081e8a1073fbad4f0512f98a1209d6df92a18eccb883a829f117df4e4628057bf2880e31c83dabfb6a41f83631005999a8dd448e195bae2c311c9d096e9b215a4d87d5cc7c51144d167b9a1874dd1e6c9540301b43647e7a517d663faa62335e38a17bb0971ad4a642ba3309d92760c4555a9a7ab9926ff43a9ef8f8c2306684578eb1f1a3b5dd9667396b9aa9a4998effdd137940ad76d1315ac088bd2b75ccd7dcf86dd248e0ec3884949e68e5dcfc8df65d9aed2afbd3bcffa45a098b31bad8282dcf6a9881f29997e0f68367f22aff31dcefaf2ab13be74a1a2ea56a092a4f802b818f366e1aeb65e4f642eb64014b727b085f3e4c84b10e71df59193492c3bd6a2b193b1c189438f09f50bdc079893bab3d521d1c29e539414097f7e31323bc68085067e5f2add034c2323ac3aba58bd6732cb3d0e95e321c4433debdfa0447bba89672a5d1208a7ed22d57f19ad6670124fc3e66892ebb6a2ae98ec3952e7e6591d72dc5b7653e6cb479c991e02895701af58c08d049430075d0a0f82b3139b7026a0d36532d54c0441ae659fdc99f9c4c3bde1f131392832ea6eab516e7d79854834fdfb4fca16ca886fed062578dd45d30bd38575e3c1d12795eb99fb5f9dca4431efbe435b615a71744c7c9716e999ec3809920841f7c0d689457143d0aaf54434e9e9e020f3ca4454fb2fe35bb7ddcb64aeecff0c850acf3bf3f5be902c09b03ab36ecd37f64b7e4ce27ac40c950b40bf04391e432f3ff6f1e3b3d5c8b16de1178a915d366c1f212ad84a7f2d6c3f84bb8f74e92f45cd3ed6d638696b1c7f8219a808c6801e0a783f2a12022403f90c2012fe836a9f14696901feedebd99fa6d16b6968ebe80846986170127d860c399a9cadbeef789cd081f8bcfca2aa22b97b95bb5ab8df9d58a2d99aede6ebab768afd8cce87c6232a04bd9193dc3f494bd43c60cb535e4bed1e8afb7b02f96df3225daccd53259f7a6e0d6d3269bf836b33cb8dab113c906936c8ae14c5222787931c73e215c4c8442818a870609a788a50cd5126ee7e06c0";
    let raw_rx_ciphertext_hex_4 = "17030300133870d0e814b644146ee4f308ae8b5a96715adb";

    // Issue: TX TLS Decryption
    //
    // TODO: Try to solve TX Direction TLS Decryption
    //
    // I really think it's a problem with the sequence number
    //
    // let tx_key_hex = "12dfa7ef1e05f99bff2bcde177505815ac6158f59a101bdfa3693a6469b95cb5";
    // let tx_iv_hex = "0557eaea974bb9241a18bde8"; 
    // let raw_tx_ciphertext_hex_0 = "1703030045fc0f30ef298c5f3fd16480e68846afd4b6f30c8eb06146accfe3d73ddcc20d7c13baadbff74d631dc07757a7b967822489e3bb94383e3496e70c2e1b2c04d37aa1ef7f1b51";
    // let raw_tx_ciphertext_hex_1 = "170303004dcc2e2ac7df7c81fd7741ae43af0e0180eccdca2891feb1e83c5868e0a795734d94e398b936b96873853d49730b03d41ef2c33b9e1425e71c34023cffeba2e38d79e76941fc1edaee1618eeebde";
    
    let aad_length = 5; // AAD length in bytes

    // Call the decrypt_data function and handle the result
    match decrypt_data(rx_key_hex, rx_iv_hex, raw_rx_ciphertext_hex_0, aad_length) {
        Ok(decrypted_data) => println!("Decrypted data (hex): {}", decrypted_data),
        Err(e) => eprintln!("Decryption error: {:?}", e),
    }
}
