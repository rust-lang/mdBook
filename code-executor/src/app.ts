// -----------------------------------------------------------------------------
// Codam Coding College, Amsterdam @ 2023.
// See README in the root project for more information.
// -----------------------------------------------------------------------------

import cors from "cors";
import express from "express";
import { Request, Response, NextFunction } from "express";
import { Execution } from "./executor";

// Globals
/*============================================================================*/

export const webserv = express();
export const port = 4242;

// Middleware
/*============================================================================*/

webserv.use(cors());
webserv.use(express.json());
webserv.use(express.urlencoded({ extended: true }));
webserv.use((err: any, req: Request, res: Response, next: NextFunction) => {
	if (err.statusCode === 400 && "body" in err)
		res.status(400).send({ status: 400, message: err.message });
});

// Routes
/*============================================================================*/

webserv.post('/playground/', (req, res) => {
	const code = req.body.code;
	const flags = req.body.flags;
	const languange = req.body.language;

	// Check request
	if(!req.is("application/json"))
		return res.status(400).json({ result: null, error: "Incorrect content type!" });
	if (code == null || languange == null || flags == null)
		return res.status(400).json({ result: null, error: "Malformed body" });

	// TODO: Check from which domain the request came from.
	// TODO: Probs add a few more checks here for unwanted requests.

	// Find module
	const module = Execution.modules[languange];
	if (module == undefined)
		return res.status(404).json({ result: null, error: "Unsupported Language!" });

	Execution.run(module, code, flags, res);

	console.log(`[Playground] [${languange}] body:`, code);
	return res.json({ result: "Request received!\n", error: null });
});


// Entry point
/*============================================================================*/

webserv.listen(port, () => {
	console.log(`[Playground] Running on: ${port}`);
});
