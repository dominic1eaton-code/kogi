import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletBankingComponent } from './wallet-banking.component';

describe('WalletBankingComponent', () => {
  let component: WalletBankingComponent;
  let fixture: ComponentFixture<WalletBankingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletBankingComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletBankingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
