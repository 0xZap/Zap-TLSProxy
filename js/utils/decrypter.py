from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend
import struct
import cryptography.exceptions

# Supondo que você tenha a chave e o IV
tx_key = bytes.fromhex('a3a6614131bf8095b9f086a7ba726af9eb4a7face8cc3126bf633e03ddf687bf')
tx_iv = bytes.fromhex('179db7738d075f5ca307d5be')
tx_sequence_number = 1

# Construa o nonce usando o IV e o número de sequência
nonce = tx_iv + struct.pack('>Q', tx_sequence_number)

# Ciphertext fornecido
ciphertext = bytes.fromhex('628f0b91e36eb0040223bf92f69f4b7c661403c95bee535d18597cc777e0df2e7ce13efa420d')

# Tentar extrair os últimos 16 bytes como tag de autenticação
if len(ciphertext) >= 16:
    tag = ciphertext[-16:]
    ciphertext = ciphertext[:-16]
else:
    raise ValueError("Ciphertext is too short to contain a valid tag.")

# Verificar o tamanho da tag
if len(tag) != 16:
    raise ValueError("A tag de autenticação deve ter 16 bytes. Tamanho atual: {}".format(len(tag)))

# Desencriptação
cipher = Cipher(algorithms.AES(tx_key), modes.GCM(nonce, tag), backend=default_backend())
decryptor = cipher.decryptor()

try:
    decrypted_data = decryptor.update(ciphertext) + decryptor.finalize()
    print(f"Mensagem desencriptada: {decrypted_data.decode('utf-8')}")
except cryptography.exceptions.InvalidTag:
    print("Falha na desencriptação: Tag inválida ou dados corrompidos.")
