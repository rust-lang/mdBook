// This is merely a dummy backend to receive the incoming requests.

const express = require("express");
const cors = require('cors');
const app = express();
const port = 4242;

/*============================================================================*/

app.use(cors());
app.use(express.json());
app.use(express.urlencoded({ extended: true }));
app.use((err, req, res, next) => {
	if (err.statusCode === 400 && "body" in err)
		res.status(400).send({ status: 400, message: err.message });
	next();
});

/*============================================================================*/

app.post('/playground', (req, res) => {
	console.log("[Playground] body:", req.body);
	console.log(req.headers);

	res.json({ result: "Request received!\n", error: null });
})

app.listen(port, () => {
	console.log(`[Playground] Running on: ${port}`);
});