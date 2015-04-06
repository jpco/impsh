#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// local includes #include "defs.h" #include "str.h"
#include "env.h"
#include "str.h"
#include "defs.h"

// self-include
#include "eval.h"

static char **jobs;
static char *job; static char **argv;

// does, as a famous man once said, "the thing".
void eval (char *cmdline)
{
/*
 * Parsing:
 * 0) [execute subjobs]
 * 1) split jobs
 *    "cat (mydir)/foo | ll" => "cat (mydir)/foo" | "ll"
 * 2) expand aliases
 *    "cat (mydir)/foo" | "ll" => "cat (mydir)/foo" | "ls -l"
 * 3) evaluate vars
 *    "cat (mydir)/foo" | "ls -l" => "cat /bin/foo" | "ls -l"
 * 4) pull apart args
 *    "cat /bin/foo" | "ls -l" => "cat" "/bin/foo" | "ls" "-l"
 * 5) execute (w/ piping/redirection)
 *
 * TODO: ADD SUBJOB SUPPORT
 */

        if (*cmdline == '\0') return;

        // STEP 1: Split jobs
        jobs = split_str (cmdline, '|');
        free (cmdline);
        int big_i;
        for (big_i=0; jobs[big_i] != NULL; big_i++) {
                job = trim_str (jobs[big_i]);
                if (*job == '\0') continue;

                // STEP 2: expand aliases
                char *jbuf;
                if ((jbuf = strchr (job, ' ')) != NULL)
                        *jbuf = '\0';

                if (has_alias (job)) {
                        char *ncmd = get_alias (job);
                        char *njob = "";
                        if (jbuf != NULL) {
                                *jbuf = ' ';
                                njob = vcombine_str ('\0', 2, ncmd, jbuf);
                        } else {
                                njob = ncmd;
                        }
                        free (job);
                        job = njob;
                } else {
                        if (jbuf != NULL)
                                *jbuf = ' ';
                }

                // STEP 3: expand vars
                jbuf = job;
                char *cbuf = jbuf;
                while ((jbuf = strchr (cbuf, '(')) != NULL) {
                        if ((cbuf = strchr (jbuf, ')')) == NULL)
                                break;
                        if (jbuf > job && *(jbuf-1) == '\\') {
                                rm_char (jbuf-1);
                                cbuf = jbuf;
                                continue;
                        }
                        *cbuf = '\0';
                        char *val;
                        if ((val = get_var (jbuf+1)) == NULL)
                                val = getenv (jbuf+1);
                        if (val == NULL) val = "";

                        *jbuf = '\0';
                        char *njob = vcombine_str ('\0', 3,
                                        job, val, cbuf+1);
                        int jbdiff = jbuf - job;
                        jbuf = njob + jbdiff;
                        free (job);
                        job = njob;
                        cbuf = jbuf;
                }

                // STEP 4: separate args
                // TODO: backslash-escaping works but "" doesn't
                argv = split_str (job, ' ');

                // STEP 5: execute (w/ piping/redirection)
                int argl;
                for (argl = 0; argv[argl] != NULL; argl++);
                if (strcmp(argv[argl-1], "&") == 0) {
                        free (argv[argl-1]);
                        argv[--argl] = NULL;
                        try_exec (argl, argv, 1);
                } else  try_exec (argl, argv, 0);
                free (argv[0]);
                free (argv);
                free (job);
        }
        free (jobs[0]);
        free (jobs);
}

void free_ceval ()
{
        free (jobs[0]);
        free (jobs);
        free (job);
        free (argv[0]);
        free (argv);
}