## LLM typing client
When practising my typing speed, especially in code based settings, the lack of varied and semantically correct programming typing apps irritated me. So I built a typing app for practising my programming.

A LLM, BigScience Bloom, is used to generate random code blocks from a pre made prompt and is sent to the typing client.

The LLM is loaded onto gpu memory and acts as a server. When a client starts typing, a pre-defined prompt with a random topic is used to generate text. 


Examples of typing experience:
```![typing](https://github.com/seancraven/generative_typing/blob/main/typing_demo.png)``` 

```![typing2](https://github.com/seancraven/generative_typing/blob/main/typing_demo2.png?raw=true)```

