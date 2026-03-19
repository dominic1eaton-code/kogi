import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletLedgerComponent } from './wallet-ledger.component';

describe('WalletLedgerComponent', () => {
  let component: WalletLedgerComponent;
  let fixture: ComponentFixture<WalletLedgerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletLedgerComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletLedgerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
