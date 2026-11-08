//! Discord's «Invita a tus amigos»: where it opens, the link it asks for
//! and hands back, and the link sent to a friend.

use rustcord_model::{Delivery, ServerId, UserId};
use rustcord_provider::{InviteFrom, InvitePlace};

use super::*;

impl Host {
    fn iv_lab(&mut self) -> ServerId {
        let lab = self
            .store
            .servers()
            .iter()
            .find(|server| server.name == "Laboratorio limn")
            .map(|server| server.id)
            .unwrap();
        self.click(&format!("server:{lab}"));
        lab
    }

    fn iv_press(&mut self, id: &str) {
        let intents = update(
            &mut self.ui,
            &mut self.store,
            &UiAction::Clicked { id: id.into() },
            self.now,
        );
        self.run(intents);
    }

    fn iv_shown(&mut self, id: &str) -> bool {
        self.frame()
            .regions
            .iter()
            .any(|region| region.id == id && region.kind == limn::HitKind::Button)
    }

    /// The invitation commands sent since `from`, answered by the simulation.
    fn iv_answer(&mut self, from: usize) -> Vec<Command> {
        let asked: Vec<Command> = self.sent[from..]
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    Command::CreateInvite { .. } | Command::OpenDirect { .. }
                )
            })
            .cloned()
            .collect();
        for command in &asked {
            for event in self.provider.manage(command.clone()) {
                self.event(event);
            }
        }
        asked
    }

    /// The link the modal shows, once made.
    fn iv_link(&mut self) -> Option<String> {
        crate::invite_modal::shown_link(&self.ui, &self.store)
            .filter(|link| self.painted().contains(link))
    }

    /// Friends outside the server: the modal's rows.
    fn iv_outsiders(&self, server: ServerId) -> Vec<UserId> {
        self.store
            .friends()
            .into_iter()
            .filter(|user| self.store.member_of(server, *user).is_none())
            .collect()
    }
}

fn opening(command: Option<&Command>) -> Option<(InvitePlace, Option<String>)> {
    match command? {
        Command::CreateInvite {
            from: InviteFrom::Opening { place, previous },
            ..
        } => Some((*place, previous.clone())),
        _ => None,
    }
}

#[test]
fn the_header_offers_it_beside_the_name_only_to_who_may_invite() {
    let mut host = Host::new();
    let lab = host.iv_lab();
    let button = format!("sm:invite-button:{lab}");
    assert!(host.iv_shown(&button), "Discord's person with a plus");
    let open = host.ui.channel().unwrap();
    let from = host.sent.len();
    host.click(&button);
    assert!(host.ui.invite.is_some());
    // For the channel open, as Discord's, saying where it was asked from.
    assert!(matches!(
        host.sent[from..].last(),
        Some(Command::CreateInvite { channel, .. }) if *channel == open
    ));
    assert_eq!(
        opening(host.sent.last()),
        Some((InvitePlace::HeaderButton, None))
    );
    host.iv_press("invite:close");
    assert!(host.ui.invite.is_none());
    // A connection that cannot make invitations has no button.
    host.store
        .provider
        .as_mut()
        .unwrap()
        .capabilities
        .create_invites = false;
    assert!(!host.iv_shown(&button));
}

#[test]
fn opening_again_hands_back_the_link_and_generating_makes_another() {
    let mut host = Host::new();
    let lab = host.iv_lab();
    let from = host.sent.len();
    host.iv_press(&format!("sm:invite-button:{lab}"));
    host.iv_answer(from);
    let link = host.iv_link().expect("the link is shown");
    // Its default lasts 30 days, as the owner's Discord.
    assert!(host
        .painted()
        .iter()
        .any(|text| text.starts_with("Tu enlace de invitación caduca en 30 días.")));
    let made = host.store.invites(lab).map_or(0, |invites| invites.len());
    host.iv_press("invite:close");
    // Again: the code made before goes with it, and the same link comes back.
    let from = host.sent.len();
    host.iv_press(&format!("sm:invite-button:{lab}"));
    assert_eq!(
        opening(host.sent.last()),
        Some((InvitePlace::HeaderButton, Some(link.clone())))
    );
    host.iv_answer(from);
    assert_eq!(host.iv_link(), Some(link.clone()));
    assert_eq!(
        host.store.invites(lab).map_or(0, |invites| invites.len()),
        made,
        "nothing new was made"
    );
    // «Generar un nuevo enlace» never hands it back.
    host.iv_press("invite:edit");
    let from = host.sent.len();
    host.iv_press("invite:generate");
    assert!(matches!(
        host.sent.last(),
        Some(Command::CreateInvite {
            from: InviteFrom::Generating(InvitePlace::HeaderButton),
            ..
        })
    ));
    host.iv_answer(from);
    let fresh = host.iv_link().unwrap();
    assert_ne!(fresh, link);
    // Copied whole, and the button says so for a moment.
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::Clicked {
            id: "invite:copy".into(),
        },
        host.now,
    );
    assert_eq!(intents, [Intent::Copy(fresh)]);
    assert!(host.painted().iter().any(|text| text == "Copiado"));
    assert!(host.ui.expire(host.now + Duration::from_secs(2)));
    assert!(host.painted().iter().any(|text| text == "Copiar"));
}

#[test]
fn a_friend_is_sent_the_link_in_their_conversation_one_at_a_time() {
    let mut host = Host::new();
    let lab = host.iv_lab();
    let outsiders = host.iv_outsiders(lab);
    assert!(
        outsiders.len() >= 2,
        "the simulation has friends outside: {outsiders:?}"
    );
    let from = host.sent.len();
    host.iv_press(&format!("sm:invite-button:{lab}"));
    host.iv_answer(from);
    let link = host.iv_link().unwrap();
    // Members are left out; the others each have their «Invitar».
    for user in host.store.friends() {
        let shown = host.iv_shown(&format!("invite:send:{user}"));
        assert_eq!(shown, outsiders.contains(&user), "{user}");
    }
    let (first, second) = (outsiders[0], outsiders[1]);
    let from = host.sent.len();
    host.iv_press(&format!("invite:send:{first}"));
    // Their conversation is opened when there is none, never shown.
    let open = host.ui.channel();
    host.iv_answer(from);
    assert_eq!(host.ui.channel(), open, "the modal stays where it is");
    let direct = host.store.direct_with(first).expect("their conversation");
    assert!(matches!(
        host.sent.last(),
        Some(Command::SendMessage { channel, content, .. })
            if *channel == direct && *content == link
    ));
    // While it is on its way, the next friend waits.
    host.iv_press(&format!("invite:send:{second}"));
    assert!(!host
        .sent
        .iter()
        .any(|command| matches!(command, Command::OpenDirect { user, .. } if *user == second)));
    assert!(!host.iv_shown(&format!("invite:send:{second}")), "waits");
    // Confirmed, it says so, and the next one can go.
    let message = host
        .store
        .timeline(direct)
        .unwrap()
        .messages()
        .last()
        .unwrap()
        .clone();
    let newest = host
        .store
        .timeline(direct)
        .and_then(|timeline| timeline.newest())
        .map_or(1, |message| message.id.0 + 1);
    let mut confirmed = (*message).clone();
    confirmed.id = rustcord_model::MessageId(newest);
    confirmed.delivery = Delivery::Sent;
    host.event(Event::MessageCreated {
        message: confirmed,
        nonce: Some(host.nonce),
    });
    assert!(host
        .painted()
        .iter()
        .any(|text| text == "Invitaciones enviadas"));
    assert!(!host.iv_shown(&format!("invite:send:{first}")), "sent once");
    assert!(host.iv_shown(&format!("invite:send:{second}")));
}

#[test]
fn a_search_finds_friends_in_the_server_and_names_them_members() {
    let mut host = Host::new();
    let lab = host.iv_lab();
    let Some(member) = host
        .store
        .friends()
        .into_iter()
        .find(|user| host.store.member_of(lab, *user).is_some())
    else {
        return;
    };
    host.iv_press(&format!("sm:invite-button:{lab}"));
    let name = host.store.display_name(member, None);
    assert!(!host.painted().iter().any(|text| text == "Miembro"));
    let intents = update(
        &mut host.ui,
        &mut host.store,
        &UiAction::InputChanged {
            id: "invite:query".into(),
            value: name.clone(),
        },
        host.now,
    );
    host.run(intents);
    let painted = host.painted();
    assert!(painted.contains(&name));
    assert!(painted.iter().any(|text| text == "Miembro"));
    assert!(!host.iv_shown(&format!("invite:send:{member}")));
}

#[test]
fn a_channels_menu_and_row_invite_to_that_channel() {
    let mut host = Host::new();
    host.iv_lab();
    let room = host.channel_named("Sala común");
    host.iv_press(&format!("cl:invite-menu:{room}"));
    assert!(matches!(
        host.sent.last(),
        Some(Command::CreateInvite { channel, .. }) if *channel == room
    ));
    assert_eq!(
        opening(host.sent.last()),
        Some((InvitePlace::ChannelMenu, None))
    );
    host.iv_press("invite:close");
    host.iv_press(&format!("cl:invite:{room}"));
    assert_eq!(
        opening(host.sent.last()),
        Some((InvitePlace::ChannelRow, None))
    );
}
