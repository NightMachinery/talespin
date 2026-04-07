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
	onmessage_handler: ((data: Record<string, unknown>) => void)[] = [];
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
			const data = JSON.parse(event.data.toString()) as Record<string, unknown>;
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
		const data_str = JSON.stringify(data);
		if (this._ws.readyState === 1) {
			this._ws.send(data_str);
		} else {
			this.message_queue.push(data_str);
		}
	}

	createRoom(name: string) {
		const normalizedName = name.trim();
		this.send({
			CreateRoom: {
				name: normalizedName
			}
		});
	}

	joinRoom(room_id: string, name: string, token: string, roomPassword?: string) {
		const normalizedName = name.trim();
		const trimmedPassword = roomPassword?.trim();
		this.send({
			JoinRoom: {
				name: normalizedName,
				room_id,
				token,
				...(trimmedPassword ? { room_password: trimmedPassword } : {})
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

	setGameMode(game_mode: 'dixit_plus' | 'stella') {
		this.send({
			SetGameMode: {
				game_mode
			}
		});
	}

	setWinCondition(win_condition: object) {
		this.send({
			SetWinCondition: {
				win_condition
			}
		});
	}

	setStorytellerLossComplement(complement: number) {
		this.send({
			SetStorytellerLossComplement: {
				complement
			}
		});
	}

	setStorytellerLossComplementAuto(enabled: boolean) {
		this.send({
			SetStorytellerLossComplementAuto: {
				enabled
			}
		});
	}

	setVotesPerGuesser(votes: number) {
		this.send({
			SetVotesPerGuesser: {
				votes
			}
		});
	}

	setCardsPerHand(cards: number) {
		this.send({
			SetCardsPerHand: {
				cards
			}
		});
	}

	setNominationsPerGuesser(cards: number) {
		this.send({
			SetNominationsPerGuesser: {
				cards
			}
		});
	}

	setBonusCorrectGuessOnThresholdCorrectLoss(enabled: boolean) {
		this.send({
			SetBonusCorrectGuessOnThresholdCorrectLoss: {
				enabled
			}
		});
	}

	setBonusDoubleVoteOnThresholdCorrectLoss(enabled: boolean) {
		this.send({
			SetBonusDoubleVoteOnThresholdCorrectLoss: {
				enabled
			}
		});
	}

	setShowVotingCardNumbers(enabled: boolean) {
		this.send({
			SetShowVotingCardNumbers: {
				enabled
			}
		});
	}

	setRoundStartDiscardCount(count: number) {
		this.send({
			SetRoundStartDiscardCount: {
				count
			}
		});
	}

	setHintChoosingTimerEnabled(enabled: boolean) {
		this.send({
			SetHintChoosingTimerEnabled: {
				enabled
			}
		});
	}

	setHintChoosingTimerDuration(seconds: number) {
		this.send({
			SetHintChoosingTimerDuration: {
				seconds
			}
		});
	}

	setCardChoosingTimerEnabled(enabled: boolean) {
		this.send({
			SetCardChoosingTimerEnabled: {
				enabled
			}
		});
	}

	setCardChoosingTimerDuration(seconds: number) {
		this.send({
			SetCardChoosingTimerDuration: {
				seconds
			}
		});
	}

	setVotingTimerEnabled(enabled: boolean) {
		this.send({
			SetVotingTimerEnabled: {
				enabled
			}
		});
	}

	setVotingTimerDuration(seconds: number) {
		this.send({
			SetVotingTimerDuration: {
				seconds
			}
		});
	}

	setForceCardChoosingTimer(enabled: boolean) {
		this.send({
			SetForceCardChoosingTimer: {
				enabled
			}
		});
	}

	setForceVotingTimer(enabled: boolean) {
		this.send({
			SetForceVotingTimer: {
				enabled
			}
		});
	}

	setVotingWrongCardDisableDistribution(distribution: number[]) {
		this.send({
			SetVotingWrongCardDisableDistribution: {
				distribution
			}
		});
	}

	setStellaBoardSize(size: number) {
		this.send({
			SetStellaBoardSize: { size }
		});
	}

	setStellaSelectionMin(count: number) {
		this.send({
			SetStellaSelectionMin: { count }
		});
	}

	setStellaSelectionMax(count: number) {
		this.send({
			SetStellaSelectionMax: { count }
		});
	}

	setStellaWordPack(words: string) {
		this.send({
			SetStellaWordPack: { words }
		});
	}

	resetStellaClue() {
		this.send({
			ResetStellaClue: {}
		});
	}

	resetStellaBoard() {
		this.send({
			ResetStellaBoard: {}
		});
	}

	forceStartNextRound() {
		this.send({
			ForceStartNextRound: {}
		});
	}

	refreshHands() {
		this.send({
			RefreshHands: {}
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

	playersChoose(cards: string[]) {
		this.send({
			PlayerChooseCards: {
				cards
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

	submitVotes(cards: string[]) {
		this.send({
			SubmitVotes: {
				cards
			}
		});
	}

	submitStellaSelection(cards: string[]) {
		this.send({
			SubmitStellaSelection: {
				cards
			}
		});
	}

	revealStellaCard(card: string) {
		this.send({
			RevealStellaCard: {
				card
			}
		});
	}

	addMsgHandler(func: (data: Record<string, unknown>) => void) {
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
