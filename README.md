# PlayedTogether

From patch 25.10 players have been getting tons of instant feedback notifications about punished players from their recent games with no easy way to check who that was or from which game. Or maybe you've seen a familiar Riot ID and can only wonder if you have ever played together with this guy before or you're just misremembering. That is until now.

This little utility has been created with exactly these use-cases in mind. Currently it is CLI only, but in the future this might change

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.70 or higher)
- A valid Riot [Development API Key](https://developer.riotgames.com/)
    - This is needed until the app recieves it's production API key
    - you need to refresh it every 24 hours and set it as an environment varible:
    ```sh
    #windows cmd
    set RGAPI_KEY=your_api_key_here

    #windows powershell
    $env:RGAPI_KEY="your_api_key_here"

    #bash and zsh
    export RGAPI_KEY="your_api_key_here"

    #fish
    set -x RGAPI_KEY "your_api_key_here"

### Install

Clone the repository and run the project:

```sh
git clone https://github.com/domahet/playedtogether.git
cd playedtogether
cargo run
```

### Usage

Currently the Riot IDs to be checked are hardcoded, so is the region in the links, this will change as the project matures. For now, you can substitute the values in lines 18-24 of the source code and then hit `cargo run` to see the results.

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

