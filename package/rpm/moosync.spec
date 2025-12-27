Summary: Moosync is a customizable desktop music player with a clean interface
Name: moosync
Version: 10.3.2
Release: 1
URL: https://github.com/Moosync/Moosync
License: GPLv3+
Group: Applications/Multimedia
ExclusiveArch: x86_64

# Dependencies required to run the app
# BuildRequires are removed because Bazel handles the build hermetically
Distribution: Fedora Linux
Vendor: Sahil Gupte
Packager: Sahil Gupte <sahilsachingupte@gmail.com>

%description
Features
 * Play audio files on your desktop.
 * Seamlessly integrate your Spotify and Youtube (including Invidious) songs.
 * Ad-free
 * Realtime lyrics
 * Scrobble your tracks on LastFM.
 * Get music recommendations directly from Spotify, Youtube and LastFM
 * Mix and match songs from different providers in a single playlist
 * Easy to use interface
 * Customizable theme engine
 * Develop own apps on top of Moosync Extension API
 * Available on Windows and Linux and MacOS

# We rely on Bazel to place the files in the build root
%files
%defattr(-,root,root,-)
# Attributes: (Mode, Owner, Group)
%attr(0755, root, root) /usr/bin/moosync
%attr(0644, root, root) /usr/share/applications/moosync.desktop