# The VALR Rusty Bot 

## Config
(via environment variables passed to the program or in a .env file - see sample):

- __API_KEY__ and __API_SECRET__: this need to be generated at valr.com with trade permissions and kept safe and secret
- __MARKET__: this is the pair e.g. BTCZAR that the bot will trade in
- __STRATEGY__: The strategy or decision-making that will be used to place sells and buys e.g. break of structure(the only option for now)

sign up at VALR: https://www.valr.com/invite/VA3HBHZ7

API Docs: https://docs.valr.com/

The __API KEY__ and __API SECRET__ are values created in the API Keys section in your account settings.
These are required for authenticated API and Web Socket calls; see https://docs.valr.com/#authentication 
for how to generate the signature and the headers 

## Execution
Use `cargo run` with a .env (containing the config) in the same directory.

## Docker
TBD

## Strategies
### Break of Structure (BOS)
This approach is looking for a high or low swing based on a certain number of price buckets,
then using BOS it determines if a buy or sell is needed. 
Currently, this outcome is only logged.