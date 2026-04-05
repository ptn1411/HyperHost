const express = require('express');
const app = express();
const port = 4200;

app.use(express.json());

// GET endpoint
app.get('/', (req, res) => {
  res.send('<h1>🚀 DevHost Node.js Test App</h1><p>GET Request successful! The server is running on port 4200.</p>');
});

app.get('/api/data', (req, res) => {
    res.json({
        message: "Hello from Node.js!",
        timestamp: new Date().toISOString(),
        status: "success"
    });
});

// POST endpoint
app.post('/api/echo', (req, res) => {
  console.log('Received POST data:', req.body);
  res.json({
    message: "POST Data received!",
    received: req.body,
    timestamp: new Date().toISOString()
  });
});

app.listen(port, '0.0.0.0', () => {
    console.log(`Test app listening at http://0.0.0.0:${port}`);
});
