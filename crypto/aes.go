package crypto

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/sha256"
)

var iv = []byte{35, 46, 57, 24, 85, 35, 24, 74, 87, 35, 88, 98, 66, 32, 14, 05}

const key_len = 32

func GenerateKey(key string) []byte {
	hash := sha256.Sum256([]byte(key))
	return hash[:key_len]
}

func Encrypt(key []byte, data []byte) ([]byte, error) {
	aes, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	cfb_enc := cipher.NewCFBEncrypter(aes, iv)
	result := make([]byte, len(data))
	cfb_enc.XORKeyStream(result, data)
	return result, nil
}

func Decrypt(key []byte, data []byte) ([]byte, error) {
	aes, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	cfb_dec := cipher.NewCFBDecrypter(aes, iv)
	result := make([]byte, len(data))
	cfb_dec.XORKeyStream(result, data)
	return result, nil
}
