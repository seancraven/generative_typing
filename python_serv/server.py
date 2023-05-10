import logging
import socket
from functools import wraps
from time import sleep
from time import time
from typing import Generator


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
        self.logger = logging.getLogger()
        self.host = "127.0.0.1"
        self.port = 5087

    @log_time
    def send_response(self, conn: socket.SocketType):
        """The server will send the prompt, as a response to the client."""
        gen = FakeGenerator()
        i = 0
        for resp in gen:
            i += 1
            msg = str(f"Generator iteration {i}")
            msg += resp
            conn.sendall(msg.encode())
            if i > 10:
                break

    def listen(self):
        """Connect to a client, and send typing prompt"""
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as soc:
            soc.bind((self.host, self.port))
            soc.listen()
            connection, _ = soc.accept()
            logging.info("Listening for connection on %s:%s", self.host, self.port)
            with connection as conn:
                # 4096 is the number of bites recived
                data = conn.recv(4096)
                logging.info(data.decode())
                self.send_response(conn)


class FakeGenerator:
    """Debugging generator, that returns a prompt, after a fixed amount of time."""

    def __iter__(self) -> Generator[str, None, None]:
        while True:
            yield self.poll_response()

    def poll_response(self) -> str:
        """Generate a response."""
        sleep(2)
        return "Some prompt"


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    server = Server()
    logging.info("Server started")
    server.listen()
