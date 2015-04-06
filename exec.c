#include <stdlib.h>
#include <stdio.h>
#include <errno.h>
#include <string.h>
#include <signal.h>

// local includes
#include "cd.h"
#include "defs.h"
#include "eval.h"

// self include
#include "exec.h"

static int pid;

// returns 1 if child is/was running,
// 0 otherwise
int sigchild (int signo)
{
        if (pid == 0) return 0;
        else {
                kill (pid, signo);
        }
}

/**
 * NOTE:
 *  - won't mash args (I don't know why it would)
 *
 * Executes the builtin functions within jpsh.
 *
 * Arguments:
 *  - argc: the number of args
 *  - argv: the arguments
 *
 * Returns:
 *  - 0 if there is no builtin with this name
 *  - 1 if the builtin successfully executed
 *  - 2 if there was an error
 */
int builtin (int argc, char **argv)
{
        if (strcmp (argv[0], "exit") == 0) {
                atexit (free_ceval);
                exit (0);
        } else if (strcmp (argv[0], "cd") == 0) {
                if (argv[1] == NULL) { // going HOME
                        if (cd (getenv("HOME")) > 0) return 2;
                        else return 1;
                }
                if (cd (argv[1]) > 0) return 2;
                else return 1;
        } else if (strcmp (argv[0], "pwd") == 0) {
                printf("%s\n", getenv ("PWD"));
                return 1;
        } else if (strcmp (argv[0], "lsvars") == 0) {
                ls_vars();
                return 1;
        } else if (strcmp (argv[0], "lsalias") == 0) {
                ls_alias();
                return 1;
        }

        return 0;
}

void printjob (int argc, char **argv, int bg)
{
        printf ("\e[0;35m");
        if (bg) printf ("(background) ");
        printf ("[%s] ", argv[0]);
        int i;
        for (i = 1; i < argc; i++) {
                printf("%s ", argv[i]);
        }
        printf("\e[0m\n");
}

void try_exec (int argc, char **argv, int bg)
{
        printjob (argc, argv, bg);

        if (!builtin (argc, argv)) {
                int success = 0;
                int err = 0;
                pid = fork();
                if (pid < 0) printf ("Fork error\n");
                else if (pid == 0) {
                        success = execvpe (argv[0], argv, environ);
                        err = errno;
                        printf ("jpsh: ");
                        if (err == 2) {
                                printf ("command '%s' not found.\n",
                                                argv[0]);
                        }
                }
                if (!bg || !success) {
                        int status = 0;
                        if (waitpid (pid, &status, 0) < 0)
                                exit (1);
                }
                pid = 0;
        }
}