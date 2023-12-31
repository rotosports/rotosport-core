# Rotosports Generator Vesting

The Generator Vesting contract progressively unlocks ROTO that can then be distributed to LP stakers via the Generator contract.

---

## InstantiateMsg

Initializes the contract with the address of the ROTO token.

```json
{
  "token_addr": "terra..."
}
```

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

#### `RegisterVestingAccounts`

Creates vesting schedules for the ROTO token. Each vesting token should have the Generator contract address as the `VestingContractAddress`. Also, each schedule will unlock tokens at a different rate according to its time duration.

Execute this message by calling the ROTO token contract address.

```json
{
  "send": {
    "contract": <VestingContractAddress>,
    "amount": "999",
    "msg": "base64-encodedStringOfWithdrawMsg"
  }
}
```

In `send.msg`, you may encode this JSON string into base64 encoding.

```json
{
  "RegisterVestingAccounts": {
    "vesting_accounts": [
      {
        "address": "terra...",
        "schedules": {
          "start_point": {
            "time": "1634125119000000000",
            "amount": "123"
          },
          "end_point": {
            "time": "1664125119000000000",
            "amount": "123"
          }
        }
      }
    ]
  }
}
```

### `claim`

Transfer vested tokens from all vesting schedules that have the same `VestingContractAddress` (address that's vesting tokens).

```json
{
  "claim": {
    "recipient": "terra...",
    "amount": "123"
  }
}
```

### `withdraw_from_active_schedule`

Withdraw tokens from active vesting schedule.  
Withdraw is possible if there is only one active vesting schedule. Active schedule's remaining amount must be greater than withdraw amount.
This endpoint terminates current active schedule (updates end_point) and creates a new one with remaining amount minus withdrawn amount.

```json
{
  "withdraw_from_active_schedule": {
    "account": "terra...",
    "recipient": "terra...",
    "withdraw_amount": "123"
  }
}
```

## QueryMsg

All query messages are described below. A custom struct is defined for each query response.

### `config`

Returns the vesting token contract address (the ROTO token address).

```json
{
  "config": {}
}
```

### `vesting_account`

Returns all vesting schedules with their details for a specific vesting recipient.

```json
{
  "vesting_account": {
    "address": "terra..."
  }
}
```

### `vesting_accounts`

Returns a paginated list of vesting schedules in chronological order. Given fields are optional.

```json
{
  "vesting_accounts": {
    "start_after": "terra...",
    "limit": 10,
    "order_by": {
      "desc": {}
    }
  }
}
```

### `available amount`

Returns the claimable amount (vested but not yet claimed) of ROTO tokens that a vesting target can claim.

```json
{
  "available_amount": {
    "address": "terra..."
  }
}
```
