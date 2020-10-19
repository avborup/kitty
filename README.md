# Kitty
Kitty is a command line interface for interacting with the [Kattis](https://open.kattis.com) platform for coding challenges. Kitty allows you to fetch problems and test and submit solutions straight from your terminal - and with more features to come!

Credit must be given where credit is due; this CLI has been inspired by the [official submission script from Kattis](https://open.kattis.com/help/submit) and the Python Kat CLI from the guys over at [Duckapple/Kat](https://github.com/Duckapple/Kat).

## Usage
To get a list of all usable commands and their description, run
```sh
$ kitty help
```
To get further information about subcommands, simply run
```sh
$ kitty help <SUBCOMMAND>
```

### Fetching
To fetch a problem, execute
```sh
$ kitty get <PROBLEM ID>
```
This will create a directory with the problem id as the name and a subdirectory called `test`, which will contain the test cases given in the problem.

You can find the problem id in the URL of the Kattis problem, an example being `ferryloading` in `https://open.kattis.com/problems/ferryloading`.

### Testing
When you have written a solution, you can run it through the test cases with
```sh
$ kitty test [PATH TO PROBLEM]
```
This will (compile if required and) run your solution, piping the content of each test sample to stdin, showing the result afterwards.

The path argument must point to the same folder that was created using `kitty get`. Note that the default value of `PATH TO PROBLEM` is the current directory.

### Submitting
When you're happy with your solution, you can attempt to submit it to Kattis. Like with the test command, call
```sh
$ kitty submit [PATH TO PROBLEM]
```
To upload your solution, Kitty needs access to your `.kattisrc` file. Run the command, and you will receive an error telling you what to do in order to set it up.

## Installation
### Cargo
You can install Kitty using cargo.
```sh
$ cargo install --git https://github.com/KongBorup/kitty
```

### Prebuilt binaries
Prebuilt binaries will be added soon enough. If you see this and no binaries have been uploaded, feel free to create an issue - then I will get around to it as soon as possible.

## Feature requests
Feel free to create a new issue if you have any problems or have any feature requests. Pull requests are also welcome.
