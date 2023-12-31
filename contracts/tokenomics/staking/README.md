# Rotosports xROTO Staking

This staking contract allows ROTO holders to stake their tokens in exchange for xROTO. The amount of ROTO they can claim later increases as accrued fees in the Maker contract get swapped to ROTO which is then sent to stakers.

---

## InstantiateMsg

Initializes the contract with the token code ID used by ROTO and the ROTO token address.

```json
{
  "token_code_id": 123,
  "deposit_token_addr": "terra..."
}
```

## ExecuteMsg

### `receive`

CW20 receive msg.

```json
{
  "receive": {
    "sender": "terra...",
    "amount": "123",
    "msg": "<base64_encoded_json_string>"
  }
}
```

#### `Enter`

Deposits ROTO in the xROTO staking contract.

Execute this message by calling the ROTO token contract and use a message like this:
```json
{
  "send": {
    "contract": <StakingContractAddress>,
    "amount": "999",
    "msg": "base64-encodedStringOfWithdrawMsg"
  }
}
```

In `send.msg`, you may encode this JSON string into base64 encoding:
```json
{
  "enter": {}
}
```

#### `leave`

Burns xROTO and unstakes underlying ROTO (initial staked amount + accrued ROTO since staking).

Execute this message by calling the xROTO token contract and use a message like this:
```json
{
  "send": {
    "contract": <StakingContractAddress>,
    "amount": "999",
    "msg": "base64-encodedStringOfWithdrawMsg"
  }
}
```

In `send.msg` you may encode this JSON string into base64 encoding:
```json
{
  "leave": {}
}
```

## QueryMsg

All query messages are described below. A custom struct is defined for each query response.

### `config`

Returns the ROTO and xROTO addresses.

```json
{
  "config": {}
}
```

### `get_total_shares`

Returns the total amount of xROTO tokens.

```json
{
  "get_total_shares": {}
}
```

### `get_total_deposit`

Returns the total amount of ROTO deposits in the staking contract.

```json
{
  "get_total_deposit": {}
}
```
