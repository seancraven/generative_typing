import logging
import os
import socket
import random
from functools import wraps
from time import sleep
from time import time
from typing import Generator

from dotenv import load_dotenv
from transformers import AutoTokenizer, AutoModelForCausalLM, GenerationConfig
import torch


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
    def __init__(self, response_generator):
        load_dotenv()
        self.host = os.environ["HOST"]
        self.port = int(os.environ["PORT"])
        self.response_generator = response_generator

    @log_time
    def send_response(self, conn: socket.SocketType, prompt: str):
        """The server will send the prompt, as a response to the client."""
        self.response_generator.reset(prompt)
        sent_resp = ""
        for resp in self.response_generator:
            sent_resp += resp
            conn.sendall(resp.encode())

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
                    # 4096 is the number of bites recived
                    data = conn.recv(4096)
                    prompt = data.decode()
                    logging.info(data.decode())
                    try:
                        self.send_response(conn, prompt)
                    except BrokenPipeError:
                        pass
                    conn.close()
                    logging.info("Connection on %s:%s closed", self.host, self.port)


class ResponseGenerator:
    """Debugging generator, that returns a prompt, after a fixed amount of time."""

    def __init__(self):
        self.file = "./python_serv/preamble.txt"

    def __iter__(self) -> Generator[str, None, None]:
        with open(self.file) as f:
            for line in f:
                sleep(0.1)
                yield line

    def reset(self, prompt: str):
        raise NotImplementedError


class LLMGenerator(ResponseGenerator):
    def __init__(self, max_len: int = 1000):
        super().__init__()
        self.llm = LLM()
        self.topics = "./python_serv/topics.txt"
        self.max_len = max_len
        self.prompt = None

    def __iter__(self) -> Generator[str, None, None]:
        assert self.prompt is not None, "Prompt must exist, call reset."
        response = self.llm.inference(self.prompt)
        response_to_send = response[len(self.prompt) :]
        prompt = response
        while len(response) < self.max_len:
            yield response_to_send
            response = self.llm.inference(prompt)
            response_to_send = response[len(prompt) :]
            prompt = response

    def reset(self, prompt: str):
        with open(self.file, "r") as f:
            _prompt = f.readlines()
        with open(self.topics, "r") as f:
            topics_list = f.readlines()

        _prompt[0] = _prompt[0].strip("\n")
        _prompt[0] += " " + (
            topics_list[random.randint(0, len(topics_list))].strip("\n") + ".\n"
        )
        prompt = "".join(_prompt)
        self.prompt = prompt
        print(prompt)


class LLM:
    """Generator to yield progressive chunks of the llm's prompt"""

    def __init__(self):
        weights_dir = "./model_weights"
        if os.path.exists(weights_dir):
            logging.debug("Local model params")
            self.model = AutoModelForCausalLM.from_pretrained(
                weights_dir, load_in_8bit=True
            )
        else:
            logging.info("Downloading model params")
            self.model = AutoModelForCausalLM.from_pretrained("bigscience/bloom-560m")
            self.model.save_pretrained(weights_dir)
        token_dir = "./tokenizer_params"
        if os.path.exists(token_dir):
            logging.debug("Local tokenizer params")
            self.tokenizer = AutoTokenizer.from_pretrained(token_dir, load_in_8bit=True)
        else:
            logging.info("Downloading tokenizer params")
            self.tokenizer = AutoTokenizer.from_pretrained("bigscience/bloom-560m")
            self.tokenizer.save_pretrained(token_dir)
        self.model.eval()
        self.generation_config = GenerationConfig(
            temperature=0.2,
            top_k=50,
            top_p=0.95,
            repetition_penalty=1.2,
            do_sample=True,
            pad_token_id=self.tokenizer.eos_token_id,
            eos_token_id=self.tokenizer.convert_tokens_to_ids(["<|endoftext|>"]),
            min_new_tokens=5,
            max_new_tokens=10,
        )
        self.device = "cuda" if torch.cuda.is_available() else "cpu"

    @log_time
    def inference(self, prompt):
        tokens = self.tokenizer.encode(prompt, return_tensors="pt")
        response = self.tokenizer.decode(
            self.model.generate(
                tokens.to(self.device),
                self.generation_config,
            )[0]
        )
        return response


if __name__ == "__main__":
    logging.basicConfig(level=logging.WARNING)
    server = Server(LLMGenerator())
    logging.info("Server started")
    server.listen()

    # with open("python_serv/preamble.txt", "r") as f:
    #     prompt = f.read()
    #
    # gen = LLMGenerator(prompt)
    # for resp in gen:
    #     print(resp, end="")
