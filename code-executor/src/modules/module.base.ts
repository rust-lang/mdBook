// -----------------------------------------------------------------------------
// Codam Coding College, Amsterdam @ 2023.
// See README in the root project for more information.
// -----------------------------------------------------------------------------

import Shell from "child_process"

/*============================================================================*/

/**
 * An execution module describes the way a language should be compiled and run.
 * 
 * For example in C you need to compile the language and then run the out file.
 */
 class ExecutionModule {
	protected code: string;
	protected flags: string;

	/**
	 * Creates a new execution module.
	 * @param code The code to execute.
	 * @param flags Additional compiler flags
	 */
	constructor(code: string, flags: string) {
		this.code = code;
		this.flags = flags;
	}

	/**
	 * Spawn a child process and 
	 */
	execute(cb: (err: Shell.ExecException, stderr: string, stdout: string) => void): void {
		cb(new Error("Invalid module"), "", "");
	}
}

export default ExecutionModule;