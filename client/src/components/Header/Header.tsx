"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";

import Icon from "@/components/Icons";
import MobileMenuButton from "@/components/MobileMenu/MobileMenuButton";
import useStore from "@/stores/store";
import { MODAL_NAMES } from "@/utils/constant";
import { cn } from "@/utils/misc";

import Button from "../WalletButton/Button";

import styles from "./styles.module.scss";

export default function Header() {
  const pathname = usePathname();
  const currentModal = useStore((state) => state.currentModal);
  const openModalByName = useStore((state) => state.openModalByName);
  const { connected: solanaWalletConnected } = useWallet();

  return (
    <header
      className={cn(
        "sticky top-0 z-30",
        currentModal === MODAL_NAMES.ENDPOINT_SELECTOR && "!z-0",
        currentModal !== null && "!z-0",
        currentModal === MODAL_NAMES.MOBILE_MENU && "!z-40"
      )}
    >
      <div className={`${styles.header}`}>
        <div className={styles.header__container}>
          <div className={styles.header__container__start}>
            <div className={styles.header__container__start__logo}>
              <Link href="/">
                <div className="flex items-center justify-center">
                  <Image
                    src="/branding/logo.png"
                    height={48}
                    width={48}
                    alt="Apollo Logo Primary"
                    className="flex"
                  />
                  <span className="ml-[-8px] text-xl font-bold text-blue-800">
                    Astrape
                  </span>
                </div>
              </Link>
            </div>
            <div className={styles.header__container__start__nav}>
              <Link
                href={"/"}
                className={`${styles.nav__icon} ${pathname === "/" ? styles.activeLink : ""}`}
              >
                <span>Home</span>
              </Link>
              <Link
                href="/mint"
                className={`${styles.nav__icon} ${pathname === "/mint" ? styles.activeLink : ""}`}
              >
                <Icon name="Provide" />
                <span>Mint</span>
              </Link>
              <Link
                href="/deposit"
                className={`${styles.nav__icon} ${pathname === "/deposit" ? styles.activeLink : ""}`}
              >
                <Icon name="Claim" />
                <span>Deposit</span>
              </Link>
              <Link
                href="/dashboard"
                className={`${styles.nav__icon} ${pathname === "/dashboard" ? styles.activeLink : ""} `}
              >
                <Icon name="Network" />
                <span>Dashboard</span>
              </Link>
            </div>
          </div>
          <div className={`${styles.header__container__end__desktop}`}>
            <Button
              theme={!solanaWalletConnected ? "primary" : "connected"}
              label="Connected"
              icon={
                !solanaWalletConnected ? (
                  <Icon name="Wallet" />
                ) : (
                  <Icon name="Tick" />
                )
              }
              hoveredIcon={<Icon name="ButtonArrow" />}
              iconPosition="left"
              onClick={() => {
                openModalByName(MODAL_NAMES.WALLET_SELECTOR);
              }}
              isLoading={currentModal === MODAL_NAMES.WALLET_SELECTOR}
              solanaWalletRequired
            />
          </div>
          <div className={`${styles.header__container__end__mobile}`}>
            <MobileMenuButton />
            <Button
              theme={!solanaWalletConnected ? "primary" : "connected"}
              icon={
                !solanaWalletConnected ? (
                  <Icon name="Wallet" />
                ) : (
                  <Icon name="Tick" />
                )
              }
              hoveredIcon={<Icon name="ButtonArrow" />}
              iconPosition="left"
              onClick={() => {
                openModalByName(MODAL_NAMES.WALLET_SELECTOR);
              }}
              solanaWalletRequired
            />
          </div>
        </div>
      </div>
    </header>
  );
}
