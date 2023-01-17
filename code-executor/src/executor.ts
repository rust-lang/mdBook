// -----------------------------------------------------------------------------
// Codam Coding College, Amsterdam @ 2023.
// See README in the root project for more information.
// -----------------------------------------------------------------------------

import { Response } from "express";
import CExecutor from "./modules/module.c";
import CPPExecutor from "./modules/module.cpp";
import ExecutionModule from "./modules/module.base";

/*============================================================================*/

export namespace Execution {
	export type ModuleEntry = {
		executor: typeof ExecutionModule;
		extensions: string;
	}

	/** Map to associate languange with the correct executionModule */
	export const modules: { [name: string]: ModuleEntry } = {
		"c": {
			executor: CExecutor,
			extensions: ".c"
		},
		"cpp": {
			executor: CPPExecutor,
			extensions: ".cpp",
		}
	};

	/**
	 * Spawns a child process for the given module and executes the code.
	 * @param module The specified module to run
	 */
	export function run(module: ModuleEntry, code: string, flags: string, response: Response) {
		try {
			const executor = new module.executor(code, flags);

			executor.execute((err, stderr, stdout) => {
				if (err)
					response.status(500).json({ result: null, error: err });
				else if (stderr != "")
					response.status(204).json({ result: stderr, error: null });
				else
					response.status(204).json({ result: stdout, error: null });
			});
		} catch (error) {
			return response.status(500).json({ result: null, error: error });
		}
		return;
	}
}
