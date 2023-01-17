// -----------------------------------------------------------------------------
// Codam Coding College, Amsterdam @ 2023.
// See README in the root project for more information.
// -----------------------------------------------------------------------------

import ExecutionModule from "./module.base";

/*============================================================================*/

class CPPExecutor extends ExecutionModule {
	constructor(code: string, flags: string) {
		super(code, flags)
	}

	/**
	 * Executes the code
	 */
	public execute(cb: (err, stderr, stdout) => void): void {

	}
}

export default CPPExecutor;