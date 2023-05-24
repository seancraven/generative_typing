import logging
import os
import socket
from functools import wraps
from time import time
from typing import Any
from typing import Tuple

import torch
from dotenv import load_dotenv
from transformers import AutoModelForCausalLM
from transformers import AutoTokenizer
from transformers import GenerationConfig


def log_time(
    func,
):
    """Time function evaluation send to consle if needed"""

    @wraps(func)
    def time_wrap(*args, **kwargs):
        start = time()
        result = func(*args, **kwargs)
        end = time()
        logging.debug("Evaluation time of %s: %s", func.__name__, end - start)
        return result

    return time_wrap


class Server:
    """Server listens for a get request this triggers inference to begin,
    sends the generative response back piecewise."""

    @log_time
    def __init__(self):
        load_dotenv()
        logging.info("Server Starting")
        self.host = os.environ["HOST"]
        self.port = int(os.environ["PORT"])
        self.tokenizer, self.model, self.generation_config = self._load_model()
        self.device = "cuda" if torch.cuda.is_available() else "cpu"

    def listen(self):
        """Connect to a client, and send typing prompt.
        Only handels sequential connections."""
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as soc:
            soc.bind((self.host, self.port))
            soc.listen()
            while True:
                connection, _ = soc.accept()
                logging.info("Listening for connection on %s:%s", self.host, self.port)
                with connection as conn:
                    prompt = conn.recv(4096).decode()
                    logging.debug("Prompt: %s", prompt)
                    for _ in range(4):
                        try:
                            resp = self.handel_request(prompt, conn)
                            logging.debug("Response: %s", resp)
                            prompt = "".join(
                                [st + "\n" for st in resp.split("\n")[-3:]]
                            )

                        except BrokenPipeError:
                            conn.close()
                            logging.info("Connection closed")
                            break

    def _load_model(
        self,
    ) -> Tuple[Any, Any, GenerationConfig]:
        """Load the model and tokenizer"""
        hf_auth_token = os.environ["HF_TOKEN"]
        checkpoint = "bigcode/starcoder"
        model = AutoModelForCausalLM.from_pretrained(
            checkpoint,
            use_auth_token=hf_auth_token,
            load_in_8bit=True,
            device_map="auto",
        )
        tokenizer = AutoTokenizer.from_pretrained(
            checkpoint, use_auth_token=hf_auth_token
        )
        generation_config = GenerationConfig(
            temperature=0.2,
            top_k=50,
            top_p=0.95,
            repetition_penalty=1.2,
            do_sample=True,
            pad_token_id=tokenizer.eos_token_id,
            eos_token_id=tokenizer.convert_tokens_to_ids(["<|endoftext|>"]),
            min_new_tokens=30,
            max_new_tokens=100,
        )
        return tokenizer, model, generation_config

    def handel_request(self, prompt: str, conn: socket.SocketType) -> str:
        """Handle the request from the client.
        Returns the message sent to the client."""
        response = self.generate(prompt)
        conn.sendall(response.encode())
        return response

    def generate(self, prompt: str) -> str:
        """LLM Code generation from prompt.
        Only returns the generated code, not the prompt."""
        inputs = self.tokenizer.encode(prompt, return_tensors="pt").to(self.device)
        output = self.model.generate(inputs, self.generation_config)
        return self.tokenizer.decode(output[0], skip_special_tokens=True)[len(prompt) :]


if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG)
    server = Server()
    server.listen()
