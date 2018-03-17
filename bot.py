import discord
import markovify

token = "INSERT_TOKEN_HERE"

model_filename = "model.json"

def send_message(msg):
        client.send_message(channel, msg)

def send_error(msg):
        send_message("ERROR: " + msg)

def save_model():
        try:
                with open(model_filename, "w") as file:
                        file.write(model.to_json())
        except:
                send_error():

def extend_model(model, text):
        model__ = markovify.Text(text, state_size=1)
        return markovify.combine(models=[model, model__])

@client.event
async def on_message(message):
	global model
	global freq
	global max_chars
	global min_chars
        global will_ping
	global channel
	# We do not want the bot to affect itself
	if message.author == client.user:
		return
	message_id = int(message.id)
	if message.content.startswith("!nonsense set freq"):
		freq = max(0, int(message.content.split()[3]))
		return
	if message.content.startswith("!nonsense get freq"):
		send_message(str(freq))
		return
	if message.content.startswith("!nonsense set maxchars"):
		max_chars = max(0, min(2000, int(message.content.split()[3])))
		return
	if message.content.startswith("!nonsense get maxchars"):
		send_message(str(max_chars))
		return
        if message.content.startswith("!nonsense set willping true"):
		will_ping = True
                return
        if message.content.startswith("!nonsense set willping false"):
		will_ping = False
                return
        if message.content.startswith("!nonsense get willping"):
                send_message(str(will_ping))
                return
	if message.content.startswith("!nonsense set channel"):
		channel__ = client.get_channel(message.content.split()[3])
		if channel__ is None:
			send_error("Invalid channel")
		else:
			channel = channel__
		return
        if message.content.startswith("!nonsense save") or
                        message_id % save_freq == 0:
                save_model()
                return
        model = extend_model(model, message.content)
	if message.content.startswith("!nonsense") or message_id % freq == 0:
		sentence = model.make_short_sentence(max_chars, min_chars)
		if sentence is None:
                        return
                if not will_ping:
                        sentence = sentence.replace("@", "<AT>")
		print("Sending message:", sentence)
		send_message(sentence)

@client.event
async def on_ready():
	print("Logged in as")
	print(client.user.name)
	print(client.user.id)
	print('------')

print("Creating client")
client = discord.Client()

try:
        with open(model_filename, "r") as file
                model = markovify.Text.from_json(file.read())
except:
        print(model_filename, "could not be openend")
        print("creating new model")
        model = markovify.Text("Hello, I am a bot.", state_size=1)
        
freq = 1
save_freq = 50
max_chars = 140
min_chars = 1
will_ping = True
channel = client.get_channel("381903453297049604")
if channel is None:
	print("ERROR: Default channel does not exist")

print("Running client")
client.run(token)
