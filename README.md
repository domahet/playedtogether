# PlayedTogether

From patch 25.10 players have been getting tons of instant feedback notifications about punished players from their recent games with no easy way to check who that was or from which game. Or maybe you've seen a familiar Riot ID and can only wonder if you have ever played together with this guy before or you're just misremembering. That is until now.

This little utility named `ptg` has been created with exactly these use-cases in mind. Currently it is CLI only, but in the future this might change

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.87.0 or higher)
- A valid Riot [Development API Key](https://developer.riotgames.com/)

    - You need to refresh a Development API Key every 24 hours. There are two ways to make this accessible to `ptg`:
    ```sh
    #option 1: use the ptg CLI directly:
    ptg --api-key your_api_key_here

    #option 2: set it as an environment variable in your preferred shell:
    #windows cmd
    set RGAPI_KEY=your_api_key_here

    #windows powershell
    $env:RGAPI_KEY="your_api_key_here"

    #bash and zsh
    export RGAPI_KEY="your_api_key_here"

    #fish
    set -x RGAPI_KEY "your_api_key_here"
    ```

### Install

With Rust installed you can install the package straight from crates.io using cargo:

```bash
cargo install --locked ptg
```

### Usage

You can easily look up two players with a command like this:
 ```sh
 ptg <Player1#GameTag> <Player2#GameTag> --region <REGION>
 ```

Since you almost always want to check another player and yourself, and most of the time you only will use a single region, both of these can be set as default, and from then on none of them need to be provided if not needed:

```sh
ptg --self <YourInGameName#YourGameTag>
ptg --default-region <Region>
ptg <Player2#GameTag>
```

For a more detailed overview of the available options, consult the --help flag, or run the utility withouth any flags or arguments:

```sh
ptg
ptg --help
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

