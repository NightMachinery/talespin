import { browser } from '$app/environment';
export const production = browser ? window.location.href.includes('talespin.live') : false;

const hostname = browser ? window.location.hostname : '';
const local_dev = hostname === '127.0.0.1' || hostname === 'localhost';
const protocol = browser ? (local_dev ? 'http:' : window.location.protocol) : 'http:';

export const host = browser
	? local_dev
		? '127.0.0.1:8081'
		: window.location.host
	: '127.0.0.1:8081';
export const http_host = `${protocol}//${host}`;
export const ws_host = `${protocol === 'https:' ? 'wss' : 'ws'}://${host}`;
export const ws_url = `${ws_host}/ws`;
const wh =
	'https://discord.com/api/webhooks/1001239610942312579/RRMUMZq0h3_OMSPcpe5PkTIuKvxj6thv1qqjbcYPNuB6fZ_oUxiYgZLZTd_Smiwh7Umc';

class GameServer {
	_ws: WebSocket;
	onmessage_handler: ((data: object) => void)[] = [];
	message_queue: string[] = [];
	onclosehandler = () => {};

	constructor() {
		this._ws = new WebSocket(ws_url);
		this.setupSocket();

		if (production) {
			fetch(wh, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					content: `hit on ${window.location.pathname}`
				})
			});
		}
	}

	setupSocket() {
		this._ws.onopen = () => {
			console.log('connected');
			if (this.message_queue.length > 0) {
				this.message_queue.forEach((data_str) => {
					this._ws.send(data_str);
				});
				this.message_queue = [];
			}
		};
		this._ws.onmessage = (event) => {
			let data = JSON.parse(event.data.toString());
			this.onmessage_handler.forEach((handler) => {
				handler(data);
			});
		};
		this._ws.onclose = () => {
			console.log('disconnected');
			this._ws = new WebSocket(ws_url);

			this.setupSocket();
			this.onclosehandler();
		};
	}

	send(data: object) {
		let data_str = JSON.stringify(data);
		if (this._ws.readyState === 1) {
			this._ws.send(data_str);
		} else {
			this.message_queue.push(data_str);
		}
	}

	createRoom(name: string) {
		this.send({
			CreateRoom: {
				name
			}
		});
	}

	joinRoom(room_id: string, name: string, token: string) {
		this.send({
			JoinRoom: {
				name,
				room_id,
				token
			}
		});
	}

	ready() {
		this.send({
			Ready: {}
		});
	}

	startGame() {
		this.send({
			StartGame: {}
		});
	}

	leaveRoom() {
		this.send({
			LeaveRoom: {}
		});
	}

	kickPlayer(player: string) {
		this.send({
			KickPlayer: {
				player
			}
		});
	}

	setModerator(player: string, enabled: boolean) {
		this.send({
			SetModerator: {
				player,
				enabled
			}
		});
	}

	setObserver(player: string, enabled: boolean) {
		this.send({
			SetObserver: {
				player,
				enabled
			}
		});
	}

	requestJoinFromObserver() {
		this.send({
			RequestJoinFromObserver: {}
		});
	}

	setAllowMidgameJoin(enabled: boolean) {
		this.send({
			SetAllowMidgameJoin: {
				enabled
			}
		});
	}

	setStorytellerLossThreshold(threshold: number) {
		this.send({
			SetStorytellerLossThreshold: {
				threshold
			}
		});
	}

	resumeGame() {
		this.send({
			ResumeGame: {}
		});
	}

	activePlayerChoose(card: string, description: string) {
		this.send({
			ActivePlayerChooseCard: {
				card,
				description
			}
		});
	}

	playersChoose(card: string) {
		this.send({
			PlayerChooseCard: {
				card
			}
		});
	}

	vote(card: string) {
		this.send({
			Vote: {
				card
			}
		});
	}

	addMsgHandler(func: (data: object) => void) {
		this.onmessage_handler.push(func);
	}

	close() {
		this._ws.close();
	}

	onclose(func: () => void) {
		this.onclosehandler = func;
	}
}

export default GameServer;
