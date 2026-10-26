# sec-js/no-weak-cipher — cases

### case: an ECB mode

```js
crypto.createCipheriv("aes-128-ecb", key, null);
```

```expect
1:1 sec-js/no-weak-cipher
```

### case: DES

```js
crypto.createDecipheriv("des-ede3", key, iv);
```

```expect
1:1 sec-js/no-weak-cipher
```

### case: RC4

```js
crypto.createCipheriv("rc4", key, null);
```

```expect
1:1 sec-js/no-weak-cipher
```

### case: createDecipher

```js
crypto.createDecipher("aes192", pw);
```

```expect
1:1 sec-js/no-weak-cipher
```
