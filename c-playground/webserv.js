// This is merely a dummy backend to receive the incoming requests.

const express = require("express");
const cors = require('cors');
const app = express();
const port = 4242;

// Middleware
/*============================================================================*/

app.use(cors());
app.use(express.json());
app.use(express.urlencoded({ extended: true }));
app.use((err, _, res, next) => {
	if (err.statusCode === 400 && "body" in err)
		res.status(400).send({ status: 400, message: err.message });
	next();
});

// Routes
/*============================================================================*/

app.post('/playground/', (req, res) => {
	const body = req.body;


	console.log(`[Playground] [${body.lang}] body:`, body.code);
	res.json({ result: "Request received!\n", error: null });
});

// Entry point
/*============================================================================*/

app.listen(port, () => {
	console.log(`[Playground] Running on: ${port}`);
});