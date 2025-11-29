# Ziron Built-in Commands

This document describes all built-in commands available in Ziron shell.

## Directory Navigation

### `cd [directory]`
Change the current directory.

- If no directory is specified, changes to the home directory (`~`)
- Supports relative and absolute paths
- Supports `~` for home directory expansion
- Updates `PWD` environment variable

**Examples:**
```bash
cd /usr/bin
cd ~/projects
cd ..
```

### `pwd`
Print the current working directory.

**Examples:**
```bash
pwd
# Output: /root/projects/ziron
```

### `pushd [directory]`
Push the current directory onto the directory stack and change to the specified directory.

- If no directory is specified, swaps the top two directories on the stack
- Maintains a stack of directories for navigation

**Examples:**
```bash
pushd /tmp
pushd /usr
```

### `popd`
Pop a directory from the directory stack and change to it.

**Examples:**
```bash
popd
```

### `dirs`
Display the directory stack.

**Examples:**
```bash
dirs
```

---

## Command Management

### `alias [name[=value]]`
Create or list command aliases.

- Without arguments: lists all aliases
- With `name=value`: creates an alias
- Aliases are expanded before command execution

**Examples:**
```bash
alias ll='ls -la'
alias
alias ll='ls -la'
```

### `unalias name`
Remove an alias.

**Examples:**
```bash
unalias ll
```

### `type command`
Show the type of a command (builtin, external, alias, or function).

**Examples:**
```bash
type cd
# Output: cd is a shell builtin

type ls
# Output: ls is /usr/bin/ls
```

### `which command`
Locate the executable file for a command.

**Examples:**
```bash
which ls
# Output: /usr/bin/ls
```

---

## Script Execution

### `source file` or `. file`
Execute commands from a file in the current shell context.

- Variables and aliases defined in the script affect the current shell
- Scripts can access and modify the current shell's environment

**Examples:**
```bash
source ~/.zironrc
. config.ziron
```

### `function name [body]`
Define a shell function.

- Functions are stored in the shell's function table
- Available for completion and execution

**Examples:**
```bash
function greet() { echo "Hello, $1!"; }
greet World
# Output: Hello, World!
```

---

## Job Control

### `jobs`
List all background jobs.

**Examples:**
```bash
jobs
# Output: [1] Running sleep 60
```

### `fg [%job]`
Bring a background job to the foreground.

- Without argument: brings the most recent job
- With `%job`: brings the specified job

**Examples:**
```bash
fg
fg %1
```

### `bg [%job]`
Resume a stopped job in the background.

**Examples:**
```bash
bg
bg %1
```

### `kill [signal] pid|%job`
Send a signal to a process or job.

- Can target processes by PID or jobs by `%job`
- Default signal is SIGTERM

**Examples:**
```bash
kill 1234
kill %1
kill -9 1234
```

### `wait [%job]`
Wait for a background job to complete.

**Examples:**
```bash
wait %1
```

---

## System Commands

### `ulimit [options]`
Display or set resource limits.

**Examples:**
```bash
ulimit
ulimit -a
```

### `umask [mask]`
Display or set the file mode creation mask.

**Examples:**
```bash
umask
umask 022
```

### `times`
Display process times.

**Examples:**
```bash
times
```

---

## Input/Output

### `echo [arguments...]`
Print arguments to stdout.

**Examples:**
```bash
echo "Hello, World!"
echo $HOME
```

### `read [variable]`
Read input from stdin into a variable.

**Examples:**
```bash
read name
read -p "Enter name: " name
```

### `printf format [arguments...]`
Formatted output similar to C's printf.

**Examples:**
```bash
printf "Hello, %s!\n" "World"
printf "%d + %d = %d\n" 2 3 5
```

---

## Testing & Control Flow

### `test expression` or `[ expression ]`
Evaluate conditional expressions.

**Examples:**
```bash
test -f file.txt
[ -d /tmp ]
[ "$var" = "value" ]
```

### `true`
Return success (exit code 0).

**Examples:**
```bash
true
echo $?
# Output: 0
```

### `false`
Return failure (exit code 1).

**Examples:**
```bash
false
echo $?
# Output: 1
```

---

## Environment Management

### `export [name[=value]]`
Set or export environment variables.

- Without arguments: lists exported variables
- With `name=value`: sets and exports a variable

**Examples:**
```bash
export PATH="/usr/bin:$PATH"
export MY_VAR="value"
export
```

### `unset name`
Unset a variable or function.

**Examples:**
```bash
unset MY_VAR
```

---

## History

### `history`
Display command history.

**Examples:**
```bash
history
```

---

## Exit

### `exit [code]`
Exit the shell.

- Without argument: exits with code 0
- With code: exits with the specified code

**Examples:**
```bash
exit
exit 1
```

---

## Notes

- All built-in commands are executed in the current shell process
- Built-ins can modify the shell's state (environment, aliases, functions, etc.)
- Some built-ins support both short and long option forms
- Error handling: most built-ins return appropriate exit codes

