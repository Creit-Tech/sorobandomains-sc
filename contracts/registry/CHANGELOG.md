# Changelog

All notable changes to this project will be documented in this file.
See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### 0.3.0 (2024-04-06)

#### Add

- Add `set_offer`, `take_offer` and `burn_offer` functions. From now users of the protocol can set buy/sale offers for
  domains, the protocol takes a fee on each sale and sends it to the `fee_taker` specified in the contract
- Add an event for function `take_offer` with the information of the transaction

### 0.2.0 (2024-03-01)

#### Add

- Add `update_tlds` function: The admin can update the allowed TLDs, this with the idea of letting the DAO decide in the
  future if they want to allow more TLDs and what will be the requirements for those new players (this comes from the
  conversation about private businesses wanting to have their own TLD).
- Add `transfer` function: Allow owners of a domain to transfer the ownership of it, once a domain is transferred it
  gets its snapshot updated. The address of the root domain it's still the same so it needs to be updated by the new
  owner.
- Add `update_address` function: Owners of a domain can update the address the domain resolves to. Only the `address`
  value is updated.
