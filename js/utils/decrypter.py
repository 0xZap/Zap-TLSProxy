from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives.kdf.hkdf import HKDFExpand
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.backends import default_backend

# Função para derivar o IV usando HKDF
def hkdf_expand_label(secret, label, context, length):
    hkdf_label = b"tls13 " + label.encode()
    info = len(hkdf_label).to_bytes(1, 'big') + hkdf_label + b'\x00' + length.to_bytes(2, 'big')
    hkdf = HKDFExpand(algorithm=hashes.SHA256(), length=length, info=info)
    return hkdf.derive(secret)

# Derivar o Nonce Implícito a partir do secret
def derive_implicit_nonce(traffic_secret):
    return hkdf_expand_label(traffic_secret, "iv", b"", 12)  # 12 bytes para o IV em TLS 1.3

# Calcular o IV final combinando implicit_nonce e explicit_nonce
def calculate_iv(implicit_nonce, sequence_number):
    explicit_nonce = sequence_number.to_bytes(8, 'big')  # 8 bytes para o sequence number
    iv = bytes(a ^ b for a, b in zip(implicit_nonce[-len(explicit_nonce):], explicit_nonce))
    return iv

# Função para desencriptar dados usando AES-GCM
def decrypt_data(key, iv, ciphertext_with_tag):
    tag = ciphertext_with_tag[-16:]
    ciphertext = ciphertext_with_tag[:-16]
    
    cipher = Cipher(algorithms.AES(key), modes.GCM(iv, tag), backend=default_backend())
    decryptor = cipher.decryptor()
    
    decrypted_data = decryptor.update(ciphertext) + decryptor.finalize()
    return decrypted_data

# Exemplo de uso:

# Seu CLIENT_TRAFFIC_SECRET_0 ou SERVER_TRAFFIC_SECRET_0
traffic_secret_hex = "35f81fc077f69699ff782db796e898789276d9a86881793e0ef552247de6ec325ce137cbcde8f40123813354f9a5188a"
sequence_number = 0  # O primeiro pacote tem o sequence number 0
encrypted_data_hex = "3a745091f0e0088df9992fb992d2d0e5e3deed72a97879cc1a90204cead4568aa05f1bf3c90b5be8ea6ed72b71413957a7e95776aca2e6d42861d859a1aca2142622bfc69036b2fcc3a2291cfae6edff845398ce9f56ebcd997197b23dcb77b70f23932df206bcaa457c168a732c19b288141ea003763a3a59f228f479af809fb67f7f8906c4f749d124164831694c9eff93d637e5d9d732f98e360745212da2702d11054a6cb0659bcc46dd2b9a8462a562b6a09fc40fd1f5efd052bed3f31284cb8d90925e795f214e76673291ae1735e480ba352813ea6df13a442898f06d04cb1a9f1a741975ef40395ba4748a6af6184f6e6589d7897632b37a0d6d0a9101450027c66e4e8217884bdc4de88b443b48367994738c5b5d8e4b80b0f10bd7902569c512670746ba95bd9b835298dc73d98e37372741e6e915d724287f92331468be511a0060d58073f40dd821ea31cf6c26d4a31c231869402492c80f0243aff2563e5a272a7668a022077bb45f3fffe79e31542e03eaf3729df13dcc2e58928e029502b20ef21d6785be42c3fc9328a11c77e70ac626b31e28af9c217be8417bf445d62613309bae00108a2600d18e2dbd7f7d53f5d6c7c0507a5e4a31b07be1d4d45baccb652bc2a9e8e9a3c703cd9a5c44934c0026b6dc19da01d42b4a525a12c4ad12e1e53e50a7c6c80fc66374324dea2c58b94e52d539680b7f136c975c6fe85240b9b992716a3e673a2cefb1153a5f0eb55e290d198663d0ff092bd4f4611babd891ba6ace55618b656ae4311846f525d3d14837e5601a2b9dce4682d38573c8775c0522d46f79ee33138dc450d1d3dc68c1211d3d3c8f0496bd2e1a85d8ab9c03ef5ba9c24fe758d59ec270b7aaca05afa082a4ce790d3511ea65c8"

# Converter os valores hexadecimais para bytes
traffic_secret = bytes.fromhex(traffic_secret_hex)
encrypted_data = bytes.fromhex(encrypted_data_hex)

# # Derivar o nonce implícito e calcular o IV
# implicit_nonce = derive_implicit_nonce(traffic_secret)
# iv = calculate_iv(implicit_nonce, sequence_number)

# # Desencriptar os dados
# decrypted_data = decrypt_data(traffic_secret, iv, encrypted_data)
# print(f"Decrypted Data: {decrypted_data}")

# TLS 1.2 - Usar os primeiros 12 bytes do ciphertext como IV
iv_tls12 = encrypted_data[:12]  # O IV é geralmente os primeiros 12 bytes
ciphertext_with_tag = encrypted_data[12:]  # O resto é o ciphertext com o tag no final

# Desencriptar usando a mesma função
decrypted_data_tls12 = decrypt_data(traffic_secret, iv_tls12, ciphertext_with_tag)
print(f"Decrypted Data (TLS 1.2): {decrypted_data_tls12}")