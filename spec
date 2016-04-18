# The Tin Shell

## ARGUMENTS
tinsh [-c configfile] [-iln] [-d [N]] [-e command|tinfile]
 -i (force interactivity)
 -l (force login shell)
 -n (do not execute)
 -d (debug level; default 1)
 -e (execute given command)
 -c (read alternate config)

## PROCESS
per-command:
 - read command
 - word splitting
 - strong var/alias resolution
 - command splitting/block gathering
 - var resolution/shell expansion
 - redirection
 - execution
 - waiting/gathering (optional)

## SYNTAX
 - commenting: '#' = line comment, '###' delimits block comments
 - piping: the same as we all know it
 - variable resolution: not the same
    - all variables, fns, binaries, statements, etc. caught in parens
        $ echo (foo); vim (which program.py)
    - nested variables are fine; executed outside-in
        $ echo (parse (opt1) (opt2))
 - blocks: C-like
    - single-line blocks declared with ':', arbitrary-line with '{ }'
        $ if (foo) == bar: ls
        $ ifx not ls -A { echo "empty!" } else { echo "not!" }
    - line-break before block declarations is NOT valid
        $ if (foo) == bar: ls
        $ ifx not ls -A {
              echo "empty!"
          } else {
              echo "not!"
          }
      but, this is valid:
        $ if (foo) == bar: ls
          else: echo "nope"
    - blockless commands
        $ if (foo) == bar, ls
    - tinsh is block-scoped: local variables first defined in block will be deleted upon leaving block
        - however, 'set foo = bar; { set foo = baz; } echo (foo)' will echo 'baz'
        - if a variable is set, by default, resetting it will change the outer variable's value
            - 'set foo = bar; : set -l foo = baz; echo (foo)' will mask the variable like in Rust
 - quoting
 - redirection
 - job control
    - (basically) POSIX compliant
        - dont do '&' on things like 'cd' because thats just silly and will be ignored

## BUILTINS
 - cd
 - fn
    $ fn foo - this, that: ls (this) (that)
    $ set foo = (fn - this, that: ls (this) (that))
 - flow control
    - if/else
        - test equality/etc
        - 'if/else test [,:{] action'
        - $ set if = 'ifx test'
    - ifx/else
        - test success of command
        - 'ifx/else command [,:{] action'
    - while/whilex
        - $ set until = 'while not'
        - $ set untilx = 'whilex not'
    - for
    - cfor
    - with
    - break [N]/continue [N]
    - return [N]
 - set/unset
 - subsh
 - exec(/eval?)
 - exit [N]
 - hash
 - test
    - string emptiness
        - isempty / nonempty
    - string comparison
        - '==', '!='
    - pathiness
        - ispath
        - isfile
        - isdir
        - isregular
        - islink
        - issymlink
        - ishardlink
    - intiness
        - isint
        - '<', '<=', '>', '>='
 - trap
 - umask
 - coproc
 - async_chain: run a block asyncronously; on completion, run a second block
    (perhaps could be as many blocks as we care to do)
    $ async_chain { update_quietly --noconfirm } { notify-send "finished update." }
 - ?, $, ., F
    - ? query a symbol's type & value
    - $ subshell
    - . run in this shell
    - F run as function (if it's not a function, error)


## FAILURE
 - there are three main kinds of failure in Rust: I/O failure, syntax failure, and command failure.
    - I/O are reported as 'errors', and are fatal by default.  The shell will (try to) handle failures if given the 'noioerr' option. (there may be other errors, but for the most part errors will be system, and especially I/O, issues.)
    - syntax failures are reported as 'warnings', and are usually handled as best as possible. They become fatal with the 'ewarning' option.
    - when commands fail (i.e., exit with a nonzero status), the shell itself does not, by default, care.  They become fatal with the 'estatus' option.


## CONFIGURATION
Config options: (all options accessable as var __tin_<value>, unless specified
                 otherwise)
 - inter:    interactive or not (readonly after startup)
 - login:    login shell or not (readonly after startup)
 - noexec:   whether we actually execute commands (readonly after startup)
 - debug:    debug output level
 - config:   as __tinrc, config file (if extant) (readonly after startup)
 - filename: if running from file, name of file (readonly after startup)
 - command:  value of '-e' arg (readonly after startup)
 - argnum:   enforce number of arguments that a builtin or function takes.
             we can't enforce this with binaries, (or can we?) but that's fine.
 - noerr:    if an error occurs, don't panic; try to recover (not recommended)
 - estatus:  if a command exits with a nonzero status, panic
 - ewarning: if a syntax warning occurs, panic
 - pipefail: if any command in a pipe chain fails, assume that error code
             (default is to assume the error code of the last command)
 - safe:    combination of 'argnum', 'estatus', 'ewarning', 'pipefail'

## FEATURES (& sources, lol)
 - look up ksh, fish, zsh, bash, tcsh
 - rust syntax
 - ion in the redox OS