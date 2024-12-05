require('dotenv').config()

const express = require("express");
const ethers = require('ethers');
const brevis = require('brevis-sdk-typescript');

const SERVER_KEY = process.env.SERVER_KEY;
const SERVER_RPC = process.env.SERVER_RPC;
const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;


const BREVIS_KEY = process.env.BREVIS_KEY;
const BREVIS_CALLBACK_ADDRESS = process.env.BREVIS_CALLBACK_ADDRESS;

const prover = new brevis.Prover('localhost:33247');
const BrevisNetwork = new brevis.Brevis('appsdkv3.brevis.network:443');

const proofReq = new brevis.ProofRequest();


const app = express();
const port = 3010;

app.get('/', async (request, response) => {
  
  console.log("Create ZK proof using Brevis")

  try {
      let provider = new ethers.JsonRpcProvider(SERVER_RPC, );
      let signer = new ethers.Wallet(SERVER_KEY, provider);

      const currentBlock = await provider.getBlockNumber();

      // Block on ethereum ~ 15 seconds 
      // We want to measure volatility on 15 minutes
      // ~ 900 blocks (15 * 60)

      let rawLogs = await provider.getLogs({
          address: CONTRACT_ADDRESS,
          topics: [],
          fromBlock: currentBlock - 900, 
          toBlock: currentBlock
      });


      // console.log(rawLogs);
      for (let index = 0; index < rawLogs.length; index++) {
        const element = rawLogs[index];  
        console.log(element);
      
        // Add the proof
        proofReq.addReceipt(
            new brevis.ReceiptData({
                tx_hash: element.transactionHash,
                fields: [
                    new brevis.Field({  // Event id
                        log_pos: 0,
                        is_topic: true,
                        field_index: 0,
                    }),
                    new brevis.Field({  // Pool id
                        log_pos: 0,
                        is_topic: true,
                        field_index: 1,
                    }),
                    new brevis.Field({  // Uniswap Pool Address
                      log_pos: 0,
                      is_topic: true,
                      field_index: 2,
                    }),
                    new brevis.Field({  // Tick value
                      log_pos: 0,
                      is_topic: true,
                      field_index: 7,
                    }),
                ],
            }),
            index,
        );

      }

      const proofRes = await prover.prove(proofReq);

      console.log('proof', proofRes.proof);

      try {
        const brevisRes = await BrevisNetwork.submit(
          proofReq, 
          proofRes, 
          11155111, 
          11155111, 
          0, 
          BREVIS_KEY, 
          BREVIS_CALLBACK_ADDRESS
        ); 

        await BrevisNetwork.wait(brevisRes.queryKey, 11155111);
    } catch (err) {
        console.error(err);
    }      

  } catch (error) {
      console.log(error);
      request.status(500).send('Error when fetching data');
  }
});

app.listen(port, () => {
  console.log(`Server is running on port ${port}`);
});