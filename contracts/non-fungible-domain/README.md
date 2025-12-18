# Non-Fungible Domain

An NFT implementation
following [SEP-0050](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0050.md) proposed by
OpenZeppelin has been adapted to the SorobanDomains protocol.

### Why did we make our own implementation?

Starting from the fact that we just like to code our own things just for the sake of it, the main reason was that the OZ
implementation is multiple times heavier than ours because is an implementation with more features than those we really
needed. The second reason was creating a simpler version for our case.

### Compatibility with OZ's implementation

This implementation tries to be as compatible as possible with the OZ implementation; We use the same events, contract
storage keys, and errors in the same places they use them, but full compatibility might not be reached based on the
updates you make to this contract. On the other hand, SEP-0050 is fully implemented in this contract, so at the
interface level, both are 90.9% compatible. We didn't include the `token_uri` method since we don't think any contract
will ever call that, and a frontend app can just get the `base_uri` and add the specific token id after that.

We suggest you use the same events and storage keys if possible. If you need to keep more data (like us with the
registry address and each token domain node), you should use extra keys instead of changing the already defined
keys/types.

### Changes before you can use it

This implementation has some updates in order to work with SorobanDomains. Here's the list of changes you might want to
do if you want to use this NFT implementation:

- `Mint` with a `TokenNode`: Assuming you would want to have a `mint` method in your NFT, you should remove the logic
  around the `token_node` method (and its parameters) because that's logic based on how the SorobanDomains Registry
  works.
- Domain checks in the "transfer" method: We call the Registry contract when doing a transfer to confirm a domain is
  still valid. You should remove this when using this contract.
- No overflow checks: The OZ implementation uses methods like `checked_sub` to check and return a `MathOverflow` error.
  In this implementation, we are ignoring all overflow checks since we use `overflow-checks = true` at the workspace
  level. If you are not using this, then you should do what the OZ implementation does.

## **Important**

This contract has not been tested.