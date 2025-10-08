from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List, Optional

from .. import Client as SpecadoClient, Message as SpecadoMessage, PromptSpec


@dataclass
class Message:
    role: str
    content: str


@dataclass
class Choice:
    message: Message
    finish_reason: str


@dataclass
class ChatCompletion:
    choices: List[Choice]


class ChatCompletions:
    def __init__(self, client: SpecadoClient):
        self._client = client

    def create(
        self,
        model: str,
        messages: List[Dict[str, str]],
        temperature: Optional[float] = None,
        **_: Any,
    ) -> ChatCompletion:
        prompt = PromptSpec(
            messages=[
                SpecadoMessage(role=message["role"], content=message["content"])
                for message in messages
            ],
            sampling={"temperature": temperature} if temperature is not None else None,
        )

        response = self._client.complete(prompt)
        message = Message(role="assistant", content=response["content"])
        choice = Choice(message=message, finish_reason=response["finish_reason"])
        return ChatCompletion(choices=[choice])


class Chat:
    def __init__(self, client: SpecadoClient):
        self.completions = ChatCompletions(client)


class OpenAI:
    def __init__(self, provider_path: str):
        self._client = SpecadoClient(provider_path)
        self.chat = Chat(self._client)
