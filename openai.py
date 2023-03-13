import os
import openai
openai.organization = "org-q78wICuK2wrGGKb3N3dTPWAZ"
openai.api_key = os.getenv("OPENAI_API_KEY")
openai.Model.list()