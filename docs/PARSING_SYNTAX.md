# Ziron Parsing Syntax

This document describes the parsing syntax and expansion features available in Ziron shell.

## Quoted Strings

### Double Quotes (`"string"`)
- Preserves spaces and special characters
- Allows variable expansion: `"Hello $USER"`
- Allows command substitution: `"Date: $(date)"`
- Escapes: `\"`, `\$`, `` \` ``, `\\`

**Examples:**
```bash
echo "Hello, World!"
echo "User: $USER"
echo "Date: $(date)"
```

### Single Quotes (`'string'`)
- Preserves all characters literally
- No variable expansion
- No command substitution
- Only escape: `\'` for single quote

**Examples:**
```bash
echo 'Hello, $USER'
echo 'Date: $(date)'
```

---

## Variable Expansion

### Basic Expansion
- `$VAR` - Expand variable
- `${VAR}` - Braced expansion (useful for concatenation)

**Examples:**
```bash
echo $HOME
echo ${HOME}/projects
```

### Advanced Expansion
- `${VAR:-default}` - Use default if VAR is unset
- `${VAR:+value}` - Use value if VAR is set
- `${VAR?error}` - Error if VAR is unset
- `${#VAR}` - Length of variable
- `${VAR:offset:length}` - Substring expansion

**Examples:**
```bash
echo ${PATH:-/usr/bin}
echo ${#HOME}
echo ${HOME:0:5}
```

### Script Arguments
- `$1`, `$2`, `$3`, ... - Individual arguments
- `$#` - Number of arguments
- `$@` - All arguments as separate words
- `$*` - All arguments as single string

**Examples:**
```bash
# In script: myscript.ziron arg1 arg2
echo $1    # arg1
echo $2    # arg2
echo $#    # 2
echo $@    # arg1 arg2
```

---

## Command Substitution

### Backtick Syntax
`` `command` `` - Execute command and substitute output

**Examples:**
```bash
echo `date`
files=`ls`
```

### Dollar-Paren Syntax
`$(command)` - Execute command and substitute output (preferred)

**Examples:**
```bash
echo $(date)
files=$(ls)
```

### Nested Substitution
Both syntaxes support nesting.

**Examples:**
```bash
echo $(echo $(date))
```

---

## Arithmetic Expansion

### Basic Arithmetic
`$((expression))` - Evaluate arithmetic expression

**Operators:**
- `+`, `-`, `*`, `/`, `%` - Basic arithmetic
- `&`, `|`, `^`, `~` - Bitwise operations
- `<<`, `>>` - Bitwise shifts
- `==`, `!=`, `<`, `>`, `<=`, `>=` - Comparisons
- `&&`, `||`, `!` - Logical operations

**Examples:**
```bash
echo $((2 + 3))        # 5
echo $((10 / 2))       # 5
echo $((5 & 3))        # 1
echo $((5 | 3))        # 7
echo $((5 == 5))       # 1 (true)
echo $((5 < 10))       # 1 (true)
echo $((5 && 3))       # 1 (true)
```

---

## Globbing (Pathname Expansion)

### Wildcards
- `*` - Match any characters
- `?` - Match single character
- `[...]` - Character class
- `[!...]` - Negated character class
- `[a-z]` - Range matching

**Examples:**
```bash
ls *.txt
ls file?.txt
ls [abc]*.txt
ls [!a-z]*.txt
```

---

## Tilde Expansion

- `~` - Home directory
- `~user` - User's home directory
- `~+` - Current directory (PWD)
- `~-` - Previous directory (OLDPWD)

**Examples:**
```bash
cd ~
cd ~/projects
cd ~+
cd ~-
```

---

## Brace Expansion

### Simple Expansion
`{a,b,c}` - Expands to: `a b c`

**Examples:**
```bash
echo {a,b,c}
# Output: a b c
```

### Range Expansion
`{1..10}` - Expands to: `1 2 3 4 5 6 7 8 9 10`

**Examples:**
```bash
echo {1..5}
# Output: 1 2 3 4 5

echo {a..e}
# Output: a b c d e
```

### Nested Expansion
`{a,{b,c}}` - Expands to: `a b c`

**Examples:**
```bash
echo {a,{b,c}}
# Output: a b c
```

### Prefix/Suffix
`prefix{a,b}suffix` - Expands to: `prefixasuffix prefixbsuffix`

**Examples:**
```bash
echo file{1,2,3}.txt
# Output: file1.txt file2.txt file3.txt
```

---

## Redirection

### Output Redirection
- `>` - Redirect stdout to file (overwrite)
- `>>` - Redirect stdout to file (append)
- `2>` - Redirect stderr to file
- `2>>` - Redirect stderr to file (append)
- `&>` - Redirect both stdout and stderr
- `n>` - Redirect file descriptor n

**Examples:**
```bash
echo "test" > output.txt
echo "more" >> output.txt
command 2> error.log
command &> all.log
```

### Input Redirection
- `<` - Redirect stdin from file
- `n<` - Redirect file descriptor n from file

**Examples:**
```bash
cat < input.txt
command < input.txt
```

### File Descriptor Duplication
- `n>&m` - Duplicate file descriptor n to m

**Examples:**
```bash
command 2>&1    # Redirect stderr to stdout
command 1>&2    # Redirect stdout to stderr
```

---

## Here-Documents

### Basic Here-Document
```bash
command << EOF
line 1
line 2
EOF
```

### Here-Document with Dash
```bash
command <<- EOF
	line 1
	line 2
EOF
```
Strips leading tabs from each line.

### Quoted Delimiter
```bash
command << 'EOF'
$VAR will not be expanded
EOF
```

---

## Here-Strings

```bash
command <<< "string"
```

**Examples:**
```bash
cat <<< "hello"
grep "test" <<< "this is a test"
```

---

## Process Substitution

### Input Process Substitution
```bash
command <(process)
```

**Examples:**
```bash
diff <(sort file1) <(sort file2)
```

### Output Process Substitution
```bash
command >(process)
```

**Examples:**
```bash
echo "test" >(tee log.txt)
```

---

## Piping

### Basic Pipes
```bash
command1 | command2
```

**Examples:**
```bash
ls -la | grep ".txt"
ps aux | grep "ziron"
```

### Multiple Pipes
```bash
command1 | command2 | command3
```

**Examples:**
```bash
cat file.txt | grep "test" | wc -l
```

---

## Background Processes

### Background Execution
```bash
command &
```

**Examples:**
```bash
sleep 60 &
jobs
fg %1
```

---

## Operator Precedence

1. Parentheses (for grouping)
2. Arithmetic operators (in order: `*`, `/`, `%`, `+`, `-`)
3. Bitwise operators (`&`, `^`, `|`)
4. Comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`)
5. Logical operators (`&&`, `||`)

---

## Notes

- All expansions happen in a specific order:
  1. Command substitution
  2. Arithmetic expansion
  3. Variable expansion
  4. Tilde expansion
  5. Brace expansion
  6. Globbing

- Quoted strings prevent most expansions (except in double quotes)
- Escaping with `\` prevents special character interpretation
- Multiple expansions can be combined

