// -----------------------------------------------------------------------------
// Codam Coding College, Amsterdam @ 2023.
// See README in the root project for more information.
// -----------------------------------------------------------------------------

import Shell from "child_process"
import ExecutionModule from "./module.base";

/*============================================================================*/

class CExecutor extends ExecutionModule {
	constructor(code: string, flags: string) {
		super(code, flags)
	}

	/**
	 * Compiles and executes the code
	 */
	public execute(cb: (err, stderr, stdout) => void): void {

		// Create file with code in it.
		// ...

		// Compile it
		Shell.exec(`gcc ${this.flags} -o`, { timeout: 10000 }, (err, stdout: string, stderr: string) => cb(err, stderr, stdout));

		// Run it
		Shell.execFile(``, { timeout: 10000 }, (err, stdout, stderr) => cb(err, stderr, stdout));
	}
}

export default CExecutor;