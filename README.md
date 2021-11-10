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

### Templates
You can define your own custom templates for your preferred programming language. If you use `kitty get`, you can add an optional parameter `--lang`, specifying what template you want to use. For example, in your kitty config directory, you can create the file `kitty/templates/template.java` containing the following code:
```java
import java.util.Scanner;

public class $FILENAME {
    public static void main(String[] args) {
        Scanner sc = new Scanner(System.in);

        sc.close();
    }
}
```
Then you can call
```sh
$ kitty get --lang java
```
which will create a file called `<PROBLEM ID>.java` that you can use.

Alternatively, you can set the default language for kitty to use so that you don't need to specify the language argument every time you fetch a problem. See the following configuration section for more.

### Configuration
Kitty does not store information about languages (compile commands, file extensions, run commands etc.) - instead it is you who must provide the languages and that information. This also means you are completely free to specify compiler flags, add new languages and so forth.

The configuration is done via a YAML file called `kitty.yml` located in your kitty configuration folder. To find the location of that, run `kitty config --location`. This repository contains an example configuration with comments describing the different configuration options: [kitty.yml](https://github.com/avborup/kitty/blob/master/kitty.yml). Feel free to simply download that file.

To see which languages kitty have picked up and how to refer to a specific language when using other kitty commands, run
```
$ kitty langs
Name       Extension
C#         cs
Go         go
Haskell    hs
Java       java
Python 3   py
Rust       rs
```

## Installation
### Installation script (Windows only)
You can use the following PowerShell command to install kitty. This will download the latest binary and a default config file.

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/avborup/kitty/master/scripts/install.ps1'))
```
### Cargo
You can install Kitty using cargo.
```sh
$ cargo install --git https://github.com/KongBorup/kitty
```

### Prebuilt binaries
You can find prebuilt binaries in the [releases section](https://github.com/KongBorup/kitty/releases) for Windows, Linux, and macOS. Once the binary is downloaded, you should be ready to go. Remember to make sure that the binary is in your PATH.

As an example, to install kitty in `/usr/local/bin` on Ubuntu, you can run the following command:
```sh
(cd /usr/local/bin && curl -L https://github.com/avborup/kitty/releases/latest/download/kitty-x86_64-unknown-linux-gnu > kitty && chmod +x kitty)
```
At this point, you should be able to freely run `kitty` anywhere on your system. If you get a permission denied error, maybe you need to give yourself access to the install folder via `sudo chown -R $(whoami) /usr/local/bin`.

## Updating
To update kitty, you can run `kitty update`.

## Feature requests
Feel free to create a new issue if you have any problems or have any feature requests. Pull requests are also welcome.
