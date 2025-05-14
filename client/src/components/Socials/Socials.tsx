import Link from "next/link";

import GithubIcon from "../Icons/SocialIcons/Github";
import XIcon from "../Icons/SocialIcons/X";

export default function Socials() {
  return (
    <div className="mx-auto flex items-center justify-center space-x-10 py-10 text-shade-mute transition">
      <Link
        href="https://x.com/Astrape_sol"
        target="_blank"
        className="transition hover:text-primary-apollo"
      >
        <XIcon />
      </Link>
      <Link
        href="https://github.com/kws1207/astrape"
        target="_blank"
        className="transition hover:text-primary-apollo"
      >
        <GithubIcon />
      </Link>
    </div>
  );
}
